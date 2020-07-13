use std::sync::Arc;

use druid::{AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Size, UnitPoint, UpdateCtx, widget::{Flex, WidgetExt}, Widget, WindowDesc};
use druid::{Data, Lens};
use druid::widget::{Checkbox, FlexParams, Label, LabelText, List, Scroll, SizedBox, ListIter};
use piet::RenderContext;

use crate::histogram::Histogram;
use crate::image_buffer::ImageBuffer;
use crate::image_edit::ImageEditor;
use std::fmt::Formatter;
use std::cell::RefCell;

mod image_edit;
mod histogram;
mod contours;
mod brushes;
mod image_buffer;
mod channels;

#[derive(Clone, Copy, PartialEq, Eq, Data, Debug)]
enum ChannelKind {
    Red,
    Green,
    Blue,
    Alpha,
    Selection,
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ChannelKind::Red => "Red",
            ChannelKind::Green => "Green",
            ChannelKind::Blue => "Blue",
            ChannelKind::Alpha => "Alpha",
            ChannelKind::Selection => "Selection",
        })
    }
}

#[derive(Clone, Debug, Data, Lens)]
struct Channel {
    name: Option<String>,
    kind: ChannelKind,
    #[lens(name = "is_visible")]
    is_visible: bool,
    #[lens(name = "is_selected")]
    is_selected: bool,
    #[lens(name = "color")]
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
    #[lens(name = "channels")]
    channels: Arc<Vec<Channel>>,
    #[lens(name = "layers")]
    layers: Arc<Vec<RefCell<Layer>>>,
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
            Checkbox::new(LabelText::Specific(Default::default()))
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
            Checkbox::new(LabelText::Specific(Default::default()))
                .lens(Layer::is_visible),
            FlexParams::default())
        .padding(5.0)
}


fn main() {
    fn ui_builder() -> impl Widget<AppData> {
        let editor = ImageEditor::new();
        let root = Flex::row()
            .with_flex_child(editor, 1.0)
            .with_child(
                SizedBox::new(
                    Flex::column()
                        .with_flex_child(
                            Scroll::new(List::new(|| make_channel_item()))
                                .vertical()
                                .lens(AppData::channels)
                            , 1.0)
                        .with_flex_child(
                            SizedBox::new(Histogram {})
                                .width(256.0)
                                .height(100.0), 1.0)
                        .with_flex_child(
                            Scroll::new(List::new(|| make_layer_item()))
                                .vertical()
                                .lens(AppData::layers)
                            , 1.0)
                ).width(256.0));
        root
    }

    let main_window = WindowDesc::new(ui_builder)
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
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}