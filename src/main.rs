use std::cell::{Cell, RefCell};
use std::sync::Arc;

use druid::{AppLauncher, Color, LocalizedString, WindowDesc};

use crate::image_buffer::ImageBuffer;
use crate::state::{AppData, Channel, ChannelKind, Layer, LayerData};
use crate::ui::make_root;

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
mod state;
mod widgets;
mod ui;

fn main() {
    let main_window = WindowDesc::new(make_root())
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