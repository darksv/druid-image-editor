use std::cell::{Cell, RefCell, RefMut};
use std::fmt::Formatter;
use std::sync::Arc;

use druid::{AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle,
            LifeCycleCtx, LocalizedString, PaintCtx, Size, UnitPoint, UpdateCtx,
            widget::{Flex, WidgetExt}, Widget, WindowDesc};
use druid::{Data, Lens};
use druid::RenderContext;
use druid::widget::{Checkbox, FlexParams, Label, LabelText, List, ListIter, Scroll, SizedBox, Slider};

use crate::channels::Matrix;
use crate::color_picker::ColorPicker;
use crate::histogram::Histogram;
use crate::image_buffer::{ImageBuffer, merge_channels};
use crate::image_edit::ImageEditor;

mod image_edit;
mod histogram;
mod contours;
mod brushes;
mod image_buffer;
mod channels;
mod tools;
mod utils;
mod ops;
mod color_picker;

#[derive(Clone, Copy, PartialEq, Eq, Data, Debug)]
enum ChannelKind {
    Red,
    Green,
    Blue,
    Alpha,
    Selection,
    HotSelection,
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ChannelKind::Red => "Red",
            ChannelKind::Green => "Green",
            ChannelKind::Blue => "Blue",
            ChannelKind::Alpha => "Alpha",
            ChannelKind::Selection => "Selection",
            ChannelKind::HotSelection => "Hot Selection",
        })
    }
}

#[derive(Clone, Debug, Data, Lens)]
struct Channel {
    name: Option<String>,
    kind: ChannelKind,
    is_visible: bool,
    is_selected: bool,
    color: Color,
}

#[derive(Clone, Debug, Data, Lens)]
struct Layer {
    name: Option<String>,
    is_selected: bool,
    is_visible: bool,
    data: LayerData,
}

#[derive(Clone, Debug, Data)]
enum LayerData {
    RasterImage(ImageBuffer),
}

impl LayerData {
    fn as_buffer(&self) -> Option<&ImageBuffer> {
        match self {
            LayerData::RasterImage(ref buff) => Some(buff),
        }
    }

    fn as_buffer_mut(&mut self) -> Option<&mut ImageBuffer> {
        match self {
            LayerData::RasterImage(ref mut buff) => Some(buff),
        }
    }
}

#[derive(Clone, Debug, Data, Lens)]
struct AppData {
    channels: Arc<Vec<Channel>>,
    layers: Arc<Vec<RefCell<Layer>>>,
    #[data(ignore)]
    dirty: Cell<bool>,
    brush_color: color_picker::Color,
    brush_size: f64,
}

impl AppData {
    pub fn layer_mut(&self, index: usize) -> RefMut<'_, Layer> {
        self.dirty.set(true);
        self.layers[index].borrow_mut()
    }

    fn channel(&self, kind: ChannelKind) -> Option<&Channel> {
        match kind {
            ChannelKind::Red => self.channels.get(0),
            ChannelKind::Green => self.channels.get(1),
            ChannelKind::Blue => self.channels.get(2),
            ChannelKind::Alpha => self.channels.get(3),
            ChannelKind::Selection => self.channels.get(4),
            ChannelKind::HotSelection => self.channels.get(5),
        }
    }

    fn is_channel_visible(&self, kind: ChannelKind) -> bool {
        self.channel(kind).map_or(false, |ch| ch.is_visible)
    }

    fn ensure_fresh(&self) {
        if !self.dirty.get() {
            return;
        }

        let layer = self.layers[0].borrow();
        let buff = layer.data.as_buffer().unwrap();
        let r = buff.channel(ChannelKind::Red).as_slice().unwrap();
        let g = buff.channel(ChannelKind::Green).as_slice().unwrap();
        let b = buff.channel(ChannelKind::Blue).as_slice().unwrap();
        let a = buff.channel(ChannelKind::Alpha).as_slice().unwrap();
        let s = buff.channel(ChannelKind::Selection);
        let hs = buff.channel(ChannelKind::HotSelection);

        let overlay = if self.is_channel_visible(ChannelKind::Selection) {
            let mut overlay = buff.channel(ChannelKind::Alpha).to_matrix();
            for y in 0..overlay.height() {
                for x in 0..overlay.width() {
                    let s = s.get(x, y);
                    let hs = hs.get(x, y);

                    match (hs, s) {
                        (255, _) => overlay.set(x, y, 96),
                        (_, 255) => overlay.set(x, y, 128),
                        _ => ()
                    }
                }
            }
            Some(overlay)
        } else {
            None
        };

        let alpha = overlay.as_ref().map(|x| x.as_slice()).unwrap_or(a);
        let zeros = Matrix::new(buff.width(), buff.height());
        let zeros = zeros.as_slice();
        let rgba = &mut *layer.data.as_buffer().unwrap().interleaved.borrow_mut();
        merge_channels(
            if self.is_channel_visible(ChannelKind::Red) { r } else { zeros },
            if self.is_channel_visible(ChannelKind::Green) { g } else { zeros },
            if self.is_channel_visible(ChannelKind::Blue) { b } else { zeros },
            alpha,
            rgba,
        );

        self.dirty.set(false);
    }
}

impl ListIter<Layer> for Arc<Vec<RefCell<Layer>>> {
    fn for_each(&self, mut cb: impl FnMut(&Layer, usize)) {
        for (index, item) in self.iter().enumerate() {
            let item = item.borrow();
            cb(&*item, index);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Layer, usize)) {
        for (index, item) in self.iter().enumerate() {
            let mut item = item.borrow_mut();
            cb(&mut *item, index);
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}

struct ChannelThumbnail;

impl Widget<Channel> for ChannelThumbnail {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Channel, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Channel, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Channel, _data: &Channel, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Channel, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Channel, _env: &Env) {
        let size = ctx.size();
        let rect = druid::Rect::from_origin_size(druid::Point::ORIGIN, size);
        ctx.fill(rect, &data.color);
        if data.is_selected {
            ctx.stroke(rect, &Color::rgba8(255, 255, 255, 255), 2.0);
        }
    }
}

struct LayerThumbnail;

impl Widget<Layer> for LayerThumbnail {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Layer, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Layer, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Layer, _data: &Layer, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Layer, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Layer, _env: &Env) {
        let size = ctx.size();
        let rect = druid::Rect::from_origin_size(druid::Point::ORIGIN, size);
        // ctx.fill(rect, &data.color);
        if data.is_selected {
            ctx.stroke(rect, &Color::rgba8(255, 255, 255, 255), 2.0);
        }
    }
}


fn make_channel_item() -> impl Widget<Channel> {
    Flex::row()
        .with_child(
            SizedBox::new(ChannelThumbnail)
                .width(32.0)
                .height(32.0)
                .border(Color::grey8(0), 1.0)
                .on_click(|_ctx, data, _| data.is_selected ^= true)
        )
        .with_flex_child(
            Label::new(|item: &Channel, _env: &_| item.name.as_ref().cloned().unwrap_or(item.kind.to_string()))
                .align_vertical(UnitPoint::LEFT)
                .expand().height(42.0)
            , 1.0)
        .with_flex_child(
            Checkbox::new(LabelText::from(""))
                .lens(Channel::is_visible),
            FlexParams::default())
        .padding(5.0)
}

fn make_layer_item() -> impl Widget<Layer> {
    Flex::row()
        .with_child(
            SizedBox::new(LayerThumbnail)
                .width(32.0)
                .height(32.0)
                .border(Color::grey8(0), 1.0)
                .on_click(|_ctx, data, _| data.is_selected ^= true)
        )
        .with_flex_child(
            Label::new(|item: &Layer, _env: &_| item.name.as_ref().cloned().unwrap_or_else(|| "New layer".into()))
                .align_vertical(UnitPoint::LEFT)
                .expand().height(42.0)
            , 1.0)
        .with_flex_child(
            Checkbox::new(LabelText::from(""))
                .lens(Layer::is_visible),
            FlexParams::default())
        .padding(5.0)
}


fn main() {
    let editor = ImageEditor::new();
    let root = Flex::row()
        .with_flex_child(editor, 1.0)
        .with_child(
            SizedBox::new(
                Flex::column()
                    .with_flex_child(
                        SizedBox::new(ColorPicker::new())
                            .lens(AppData::brush_color), 1.0,
                    )
                    .with_flex_child(
                        SizedBox::new(Slider::new().with_range(0.0, 100.0))
                            .width(256.0)
                            .padding(5.0)
                            .lens(AppData::brush_size)
                        , 1.0,
                    )
                    .with_flex_child(
                        Scroll::new(List::new(make_channel_item))
                            .vertical()
                            .lens(AppData::channels), 1.0)
                    .with_flex_child(
                        SizedBox::new(Histogram {})
                            .width(256.0)
                            .height(100.0), 1.0)
                    .with_flex_child(
                        Scroll::new(List::new(make_layer_item))
                            .vertical()
                            .lens(AppData::layers)
                        , 1.0)
            ).width(256.0)
        );

    let main_window = WindowDesc::new(root)
        .title(LocalizedString::new("Maditor"))
        .window_size((1378.0, 768.0));

    let data = AppData {
        channels: Arc::new(
            vec![
                Channel { name: Some("Red".to_string()), kind: ChannelKind::Red, is_selected: false, is_visible: true, color: Color::rgb8(255, 0, 0) },
                Channel { name: Some("Green".to_string()), kind: ChannelKind::Green, is_selected: true, is_visible: true, color: Color::rgb8(0, 255, 0) },
                Channel { name: Some("Blue".to_string()), kind: ChannelKind::Blue, is_selected: false, is_visible: true, color: Color::rgb8(0, 0, 255) },
                Channel { name: Some("Alpha".to_string()), kind: ChannelKind::Alpha, is_selected: false, is_visible: true, color: Color::rgb8(0, 0, 0) },
                Channel { name: Some("Selection".to_string()), kind: ChannelKind::Selection, is_selected: false, is_visible: true, color: Color::rgb8(0, 0, 0) },
            ]),
        layers: Arc::new(
            vec![RefCell::new(Layer {
                name: None,
                is_selected: true,
                is_visible: true,
                data: LayerData::RasterImage(ImageBuffer::from_file("image.jpg").unwrap()),
            })]
        ),
        dirty: Cell::new(true),
        brush_color: color_picker::Color::new(),
        brush_size: 1.0,
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}