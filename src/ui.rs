use druid::{Color, UnitPoint, Widget, WidgetExt};
use druid::widget::{Checkbox, Flex, FlexParams, Label, LabelText, List, Scroll, SizedBox, Slider};

use crate::color_picker::ColorPicker;
use crate::histogram::Histogram;
use crate::image_edit::ImageEditor;
use crate::state::{AppData, Channel, Layer};
use crate::widgets::{ChannelThumbnail, LayerThumbnail};

fn make_channel_item() -> impl Widget<Channel> {
    Flex::row()
        .with_child(
            SizedBox::new(ChannelThumbnail)
                .width(32.0)
                .height(32.0)
                .border(Color::grey8(0), 1.0)
                .on_click(|_ctx, data: &mut Channel, _| data.is_selected ^= true)
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
                .on_click(|_ctx, data: &mut Layer, _| data.is_selected ^= true)
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

pub(crate) fn make_root() -> impl Widget<AppData> {
    Flex::row()
        .with_flex_child(ImageEditor::new(), 1.0)
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
        )
}
