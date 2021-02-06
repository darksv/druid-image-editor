use druid::{Point, Affine};
use crate::{AppData, ChannelKind};
use crate::brushes::{BasicBrush, Brush};
use crate::utils::interpolate_points;

pub(crate) trait Tool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData);
    fn mouse_down(&mut self, pos: Point, transform: Affine, data: &AppData);
    fn mouse_up(&mut self, transform: Affine, data: &AppData);
}

pub struct DrawTool {
    brush_size: u32,
}

impl DrawTool {
    pub(crate) fn new(brush_size: u32) -> Self {
        DrawTool { brush_size }
    }
}

impl Tool for DrawTool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData) {
        let transform = transform.inverse();
        let begin = transform * previous_pos;
        let end = transform * pos;

        for index in 0..4 {
            if !data.channels[index].is_selected {
                continue;
            }

            let mut layer = data.layer_mut(0);
            let image = layer.data.as_buffer_mut().unwrap();
            let kind = data.channels[index].kind;
            interpolate_points(begin, end, |p| {
                BasicBrush::new(self.brush_size).apply(
                    image.channel_mut(kind),
                    p.x as u32,
                    p.y as u32,
                );
            });
        }
    }

    fn mouse_down(&mut self, pos: Point, transform: Affine, data: &AppData) {
        let transform = transform.inverse();
        let p = transform * pos;

        for index in 0..4 {
            if !data.channels[index].is_selected {
                continue;
            }

            BasicBrush::new(self.brush_size).apply(
                data.layers[0].borrow_mut().data.as_buffer_mut().unwrap().channel_mut(data.channels[index].kind),
                p.x as u32,
                p.y as u32,
            );
        }
    }

    fn mouse_up(&mut self, _transform: Affine, _data: &AppData) {}
}


pub(crate) struct BrushSelectionTool {
    brush_size: u32,
}

impl BrushSelectionTool {
    pub(crate) fn new(brush_size: u32) -> Self {
        Self { brush_size }
    }
}

impl Tool for BrushSelectionTool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData) {
        let transform = transform.inverse();
        let begin = transform * previous_pos;
        let end = transform * pos;

        let mut layer = data.layer_mut(0);
        let image = layer.data.as_buffer_mut().unwrap();
        interpolate_points(begin, end, |p| {
            BasicBrush::new(self.brush_size).apply(
                image.channel_mut(ChannelKind::HotSelection),
                p.x as u32,
                p.y as u32,
            );
        });
    }

    fn mouse_down(&mut self, _pos: Point, _transform: Affine, _data: &AppData) {}

    fn mouse_up(&mut self, _transform: Affine, data: &AppData) {
        let mut layer = data.layer_mut(0);
        let (mut sel, mut hot_sel) = layer.data.as_buffer_mut().unwrap().selection_mut();

        for y in 0..sel.height() {
            for x in 0..sel.width() {
                sel.set(x, y, sel.get(x, y).saturating_add(hot_sel.get(x, y)));
                hot_sel.set(x, y, 0);
            }
        }
    }
}

pub(crate) struct ShapeSelectionTool {
    start_moving_pos: Option<Point>,
    end_moving_pos: Option<Point>,
}

impl ShapeSelectionTool {
    pub(crate) fn new() -> Self {
        Self {
            start_moving_pos: None,
            end_moving_pos: None
        }
    }
}


impl Tool for ShapeSelectionTool {
    fn mouse_move(&mut self, pos: Point, _previous_pos: Point, _transform: Affine, _data: &AppData) {
        self.end_moving_pos = Some(pos);
    }

    fn mouse_down(&mut self, pos: Point, _transform: Affine, _data: &AppData) {
        self.start_moving_pos = Some(pos);
    }

    fn mouse_up(&mut self, transform: Affine, data: &AppData) {
        let transform = transform.inverse();
        let start = transform * self.start_moving_pos.unwrap();
        let end = transform * self.end_moving_pos.unwrap();
        let x1 = (start.x.min(end.x)) as u32;
        let x2 = (start.x.max(end.x)) as u32;
        let y1 = (start.y.min(end.y)) as u32;
        let y2 = (start.y.max(end.y)) as u32;

        let mut layer = data.layer_mut(0);
        let mut v = layer.data.as_buffer_mut().unwrap().channel_mut(ChannelKind::Selection);
        for y in y1..=y2 {
            for x in x1..=x2 {
                v.set(x, y, 255);
            }
        }
    }
}