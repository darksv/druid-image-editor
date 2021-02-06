use druid::{Point, Affine};
use crate::AppData;
use crate::brushes::{BasicBrush, Brush};
use crate::utils::interpolate_points;

pub(crate) trait Tool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData);
    fn mouse_down(&mut self, pos: Point, transform: Affine, data: &AppData);
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
}