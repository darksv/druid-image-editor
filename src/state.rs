use std::cell::{Cell, RefCell, RefMut};
use std::fmt::Formatter;
use std::sync::Arc;

use druid::{Color, Data, Lens};

use crate::channels::Matrix;
use crate::color_picker;
use crate::image_buffer::{merge_channels, ImageBuffer};

#[derive(Clone, Copy, PartialEq, Eq, Data, Debug)]
pub(crate) enum ChannelKind {
    Red,
    Green,
    Blue,
    Alpha,
    Selection,
    HotSelection,
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ChannelKind::Red => "Red",
                ChannelKind::Green => "Green",
                ChannelKind::Blue => "Blue",
                ChannelKind::Alpha => "Alpha",
                ChannelKind::Selection => "Selection",
                ChannelKind::HotSelection => "Hot Selection",
            }
        )
    }
}

#[derive(Clone, Debug, Data, Lens)]
pub(crate) struct Channel {
    pub(crate) name: Option<String>,
    pub(crate) kind: ChannelKind,
    pub(crate) is_visible: bool,
    pub(crate) is_selected: bool,
    pub(crate) color: Color,
}

#[derive(Clone, Debug, Data, Lens)]
pub(crate) struct Layer {
    pub(crate) name: Option<String>,
    pub(crate) is_selected: bool,
    pub(crate) is_visible: bool,
    pub(crate) data: LayerData,
}

#[derive(Clone, Debug, Data)]
pub(crate) enum LayerData {
    RasterImage(ImageBuffer),
}

impl LayerData {
    pub(crate) fn as_buffer(&self) -> Option<&ImageBuffer> {
        match self {
            LayerData::RasterImage(ref buff) => Some(buff),
        }
    }

    pub(crate) fn as_buffer_mut(&mut self) -> Option<&mut ImageBuffer> {
        match self {
            LayerData::RasterImage(ref mut buff) => Some(buff),
        }
    }
}

#[derive(Clone, Debug, Data, Lens)]
pub(crate) struct AppData {
    pub(crate) channels: Arc<Vec<Channel>>,
    pub(crate) layers: Arc<Vec<RefCell<Layer>>>,
    #[data(ignore)]
    pub(crate) dirty: Cell<bool>,
    pub(crate) brush_color: color_picker::Color,
    pub(crate) brush_size: f64,
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

    pub(crate) fn ensure_fresh(&self) {
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
                        _ => (),
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
        #[rustfmt::skip]
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
