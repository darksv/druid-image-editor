use std::ops::Neg;

use druid::{Affine, Color, Modifiers, PaintCtx, Point, Rect, RenderContext, Vec2};
use druid::kurbo::Circle;
use druid::piet::StrokeStyle;

use crate::brushes::{BasicBrush, Brush};
use crate::state::{AppData, ChannelKind};
use crate::utils::interpolate_points;

pub(crate) trait Tool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData);
    fn mouse_down(&mut self, pos: Point, transform: Affine, data: &AppData);
    fn mouse_up(&mut self, transform: Affine, data: &AppData);
    fn wheel(&mut self, pos: Point, delta: Vec2, mods: Modifiers);
    fn overlay(&mut self, ctx: &mut PaintCtx, pos: Point, scale: f64);
}

pub struct DrawTool {
    brush_size: u32,
    color: [u8; 4],
}

impl DrawTool {
    pub(crate) fn new(brush_size: u32, color: [u8; 4]) -> Self {
        DrawTool { brush_size, color }
    }
}

impl Tool for DrawTool {
    fn mouse_move(&mut self, pos: Point, previous_pos: Point, transform: Affine, data: &AppData) {
        let transform = transform.inverse();
        let begin = transform * previous_pos;
        let end = transform * pos;

        for index in 0..4 {
            let mut layer = data.layer_mut(0);
            let image = layer.data.as_buffer_mut().unwrap();
            let kind = data.channels[index].kind;
            interpolate_points(begin, end, |p| {
                BasicBrush::new(self.brush_size, self.color[index]).apply(
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
            BasicBrush::new(self.brush_size, self.color[index]).apply(
                data.layers[0].borrow_mut().data.as_buffer_mut().unwrap().channel_mut(data.channels[index].kind),
                p.x as u32,
                p.y as u32,
            );
        }
    }

    fn mouse_up(&mut self, _transform: Affine, _data: &AppData) {}

    fn wheel(&mut self, _pos: Point, _delta: Vec2, _mods: Modifiers) {}

    fn overlay(&mut self, ctx: &mut PaintCtx, pos: Point, scale: f64) {
        ctx.with_save(|ctx| {
            let c = Color::rgb8(90, 100, 20);
            ctx.stroke(Circle::new(pos, (self.brush_size as f64) / 2.0 * scale), &c, 1.0);
        });
    }
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
            BasicBrush::new(self.brush_size, 255).apply(
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

    fn wheel(&mut self, _pos: Point, _delta: Vec2, _mods: Modifiers) {}

    fn overlay(&mut self, _ctx: &mut PaintCtx, _pos: Point, _scale: f64) {}
}

pub(crate) struct ShapeSelectionTool {
    pub(crate) start_moving_pos: Option<Point>,
    pub(crate) end_moving_pos: Option<Point>,
}

impl ShapeSelectionTool {
    pub(crate) fn new() -> Self {
        Self {
            start_moving_pos: None,
            end_moving_pos: None,
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

    fn wheel(&mut self, _pos: Point, _delta: Vec2, _mods: Modifiers) {}

    fn overlay(&mut self, ctx: &mut PaintCtx, _pos: Point, _scale: f64) {
        ctx.with_save(|ctx| {
            let c = Color::rgb8(0, 0, 0);
            let mut ss = StrokeStyle::new();
            ss.set_dash(vec![3.0, 1.0], 0.0);

            ctx.stroke_styled(
                Rect::from_points(self.start_moving_pos.unwrap(), self.end_moving_pos.unwrap()),
                &c,
                1.0,
                &ss,
            );
        });
    }
}

pub(crate) struct MovingTool {
    offset_x: f64,
    offset_y: f64,
    start_moving_pos: Point,
    start_offset_x: f64,
    start_offset_y: f64,
    scale: f64,
}

impl MovingTool {
    pub(crate) fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            start_moving_pos: Default::default(),
            start_offset_x: 0.0,
            start_offset_y: 0.0,
            scale: 1.0,
        }
    }

    pub(crate) fn transform(&self) -> Affine {
        Affine::new([self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y])
    }

    pub(crate) fn scale(&self) -> f64 {
        self.scale
    }
}

impl Tool for MovingTool {
    fn mouse_move(&mut self, pos: Point, _previous_pos: Point, _transform: Affine, _data: &AppData) {
        let image_pos_x = self.start_moving_pos.x - self.start_offset_x;
        let image_pos_y = self.start_moving_pos.y - self.start_offset_y;
        self.offset_x = pos.x - image_pos_x;
        self.offset_y = pos.y - image_pos_y;
    }

    fn mouse_down(&mut self, pos: Point, _transform: Affine, _data: &AppData) {
        self.start_moving_pos = pos;
        self.start_offset_x = self.offset_x;
        self.start_offset_y = self.offset_y;
    }

    fn mouse_up(&mut self, _transform: Affine, _data: &AppData) {}

    fn wheel(&mut self, pos: Point, delta: Vec2, mods: Modifiers) {
        match (mods.ctrl(), mods.alt(), mods.shift()) {
            (true, false, false) => {
                let new_scale = self.scale * delta.y.neg().signum().exp();

                // From formula:
                // (cursor_x - old_offset_x) / old_scale =
                // (cursor_x - new_offset_x) / new_scale
                // FIXME: cleanup
                self.offset_x = -(pos.x * new_scale - new_scale * self.offset_x - pos.x * self.scale) / self.scale;
                self.offset_y = -(pos.y * new_scale - new_scale * self.offset_y - pos.y * self.scale) / self.scale;
                self.scale = new_scale;
            }
            (false, false, false) => {
                self.offset_y += delta.y.neg();
            }
            (false, false, true) => {
                self.offset_x += delta.y.neg();
            }
            _ => ()
        }
    }

    fn overlay(&mut self, _ctx: &mut PaintCtx, _pos: Point, _scale: f64) {}
}

pub(crate) enum ToolRef<'a> {
    Owned(Box<dyn Tool>),
    Ref(&'a mut dyn Tool),
}

impl<'a> ToolRef<'a> {
    #[allow(unused)]
    fn as_ref(&'a self) -> &'a dyn Tool {
        match self {
            ToolRef::Owned(ref x) => x.as_ref(),
            ToolRef::Ref(tool) => *tool,
        }
    }

    pub(crate) fn as_mut(&'a mut self) -> &'a mut dyn Tool {
        match self {
            ToolRef::Owned(ref mut x) => x.as_mut(),
            ToolRef::Ref(tool) => *tool,
        }
    }
}