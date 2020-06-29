use std::sync::Arc;

use druid::{AppLauncher, Color, LocalizedString, UnitPoint, widget::{Flex, WidgetExt}, Widget, WindowDesc, LifeCycle, EventCtx, PaintCtx, LifeCycleCtx, BoxConstraints, Size, LayoutCtx, Event, Env, UpdateCtx, Rect};
use druid::{Data, Lens};
use druid::widget::{Checkbox, FlexParams, Label, LabelText, List, Scroll, SizedBox};
use piet::RenderContext;
use crate::histogram::Histogram;
use crate::image_buffer::ImageBuffer;
use crate::image_edit::ImageEditor;

mod image_edit;
mod histogram;
mod contours;
mod brushes;
mod image_buffer;
mod layers;

#[derive(Clone, Debug, Data, Lens)]
struct Layer {
    name: String,
    #[lens(name = "is_visible")]
    is_visible: bool,
    #[lens(name = "color")]
    color: Color,
}

#[derive(Clone, Debug, Data, Lens)]
struct AppData {
    #[lens(name = "layers")]
    layers: Arc<Vec<Layer>>,
    image: ImageBuffer,
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
        ctx.fill(druid::Rect::from_origin_size(druid::Point::ORIGIN, size), &data.color);
    }
}

fn make_layer_item() -> impl Widget<Layer> {
    Flex::row()
        .with_child(
            SizedBox::new(LayerThumbnail)
                .width(32.0)
                .height(32.0)
                .border(Color::grey8(0), 1.0)
        )
        .with_flex_child(
            Label::new(|item: &Layer, _env: &_| item.name.clone())
                .align_vertical(UnitPoint::LEFT)
                .expand().height(42.0)
            , 1.0)
        .with_flex_child(
            Checkbox::new(LabelText::Specific(Default::default()))
                .lens(Layer::is_visible),
            FlexParams::default())

        .padding(5.0)
        .background(Color::rgb(0.5, 0.5, 0.5))
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
                            Scroll::new(List::new(|| make_layer_item()))
                                .vertical()
                                .lens(AppData::layers)
                            , 1.0)
                        .with_flex_child(
                            SizedBox::new(Histogram {})
                                .width(256.0)
                                .height(100.0), 1.0)
                ).width(256.0));
        root
    }

    let main_window = WindowDesc::new(ui_builder)
        .title(LocalizedString::new("Maditor"))
        .window_size((1378.0, 768.0));

    let data = AppData {
        layers: Arc::new(
            vec![
                Layer { name: "Red".to_string(), is_visible: true, color: Color::rgb8(255, 0, 0) },
                Layer { name: "Green".to_string(), is_visible: true, color: Color::rgb8(0, 255, 0) },
                Layer { name: "Blue".to_string(), is_visible: true, color: Color::rgb8(0, 0, 255) },
                Layer { name: "Alpha".to_string(), is_visible: true, color: Color::rgb8(0, 0, 0) },
            ]),
        image: ImageBuffer::from_file("image.jpg").unwrap(),
    };
    AppLauncher::with_window(main_window)
        .use_simple_logger()
        .launch(data)
        .expect("launch failed");
}