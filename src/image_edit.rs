use std::ops::Neg;

use druid::{Affine, BoxConstraints, Code, Cursor, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget};
use kurbo::Circle;
use piet::{InterpolationMode, StrokeStyle};

use crate::{AppData, ChannelKind};
use crate::brushes::{BasicBrush, Brush};
use crate::image_buffer::merge_channels;
use crate::channels::Matrix;

pub struct ImageEditor {
    interpolation: InterpolationMode,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    mouse_position: Point,
    previous_mouse_position: Point,
    is_mouse_down: bool,
    state: EditorState,
    start_moving_pos: Point,
    start_offset_x: f64,
    start_offset_y: f64,
    end_moving_pos: Point,
    brush_size: u32,
}

enum EditorState {
    Drawing,
    Moving,
    ShapeSelection,
    BrushSelection,
}

impl ImageEditor {
    pub fn new() -> Self {
        ImageEditor {
            interpolation: InterpolationMode::NearestNeighbor,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            mouse_position: Default::default(),
            previous_mouse_position: Default::default(),
            is_mouse_down: false,
            state: EditorState::Drawing,
            start_moving_pos: Default::default(),
            end_moving_pos: Default::default(),
            start_offset_x: 0.0,
            start_offset_y: 0.0,
            brush_size: 1,
        }
    }

    fn make_transform(&self) -> Affine {
        Affine::new([self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y])
    }
}

mod bresenham {
    // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#Algorithm_for_integer_arithmetic

    fn plot_line_low(x0: i32, y0: i32, x1: i32, y1: i32, mut f: impl FnMut(i32, i32)) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        let mut d = 2 * dy - dx;
        let mut y = y0;
        for x in x0..=x1 {
            f(x, y);
            if d > 0 {
                y += yi;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }
    }

    fn plot_line_high(x0: i32, y0: i32, x1: i32, y1: i32, mut f: impl FnMut(i32, i32)) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }
        let mut d = 2 * dx - dy;
        let mut x = x0;
        for y in y0..=y1 {
            f(x, y);
            if d > 0 {
                x += xi;
                d -= 2 * dy;
            }
            d += 2 * dx;
        }
    }

    pub(crate) fn plot_line(x0: i32, y0: i32, x1: i32, y1: i32, f: impl FnMut(i32, i32)) {
        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                plot_line_low(x1, y1, x0, y0, f);
            } else {
                plot_line_low(x0, y0, x1, y1, f);
            }
        } else {
            if y0 > y1 {
                plot_line_high(x1, y1, x0, y0, f);
            } else {
                plot_line_high(x0, y0, x1, y1, f);
            }
        }
    }
}

fn interpolate_points(begin: Point, end: Point, mut f: impl FnMut(Point)) {
    let (begin, end) = if begin.x < end.x { (begin, end) } else { (end, begin) };

    let x0 = begin.x as i32;
    let y0 = begin.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    bresenham::plot_line(x0, y0, x1, y1, |x, y| f(Point::new(x as f64, y as f64)));
}

#[allow(unused)]
fn gaussian(bytes: &[u8], width: usize, height: usize, out: &mut [u8]) {
    for y in 0..height {
        for x in 1..width - 1 {
            out[y * width + x] =
                bytes[y * width + x - 1] / 4
                    + bytes[y * width + x] / 4
                    + bytes[y * width + x + 1] / 4;
        }
    }
    unsafe {
        for y in 1..height - 1 {
            for x in 0..width {
                *out.get_unchecked_mut(y * width + x) =
                    *bytes.get_unchecked((y - 1) * width + x) / 4
                        + *bytes.get_unchecked(y * width + x) / 4
                        + *bytes.get_unchecked((y + 1) * width + x) / 4;
            }
        }
    }
}

impl Widget<AppData> for ImageEditor {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseMove(e) => {
                ctx.request_focus();

                self.previous_mouse_position = self.mouse_position;
                self.mouse_position = e.pos;

                if self.is_mouse_down {
                    match self.state {
                        EditorState::Drawing => {
                            let transform = self.make_transform().inverse();
                            let begin = transform * self.previous_mouse_position;
                            let end = transform * self.mouse_position;

                            for index in 0..4 {
                                if !data.channels[index].is_selected {
                                    continue;
                                }

                                let mut layer = data.layers[0].borrow_mut();
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
                        EditorState::Moving => {
                            let image_pos_x = self.start_moving_pos.x - self.start_offset_x;
                            let image_pos_y = self.start_moving_pos.y - self.start_offset_y;
                            self.offset_x = e.pos.x - image_pos_x;
                            self.offset_y = e.pos.y - image_pos_y;
                        }
                        EditorState::ShapeSelection => {
                            self.end_moving_pos = self.mouse_position;
                        }
                        EditorState::BrushSelection => {
                            let transform = self.make_transform().inverse();
                            let begin = transform * self.previous_mouse_position;
                            let end = transform * self.mouse_position;

                            let mut layer = data.layers[0].borrow_mut();
                            let image = layer.data.as_buffer_mut().unwrap();
                            interpolate_points(begin, end, |p| {
                                BasicBrush::new(self.brush_size).apply(
                                    image.channel_mut(ChannelKind::HotSelection),
                                    p.x as u32,
                                    p.y as u32,
                                );
                            });
                        }
                    }
                }

                ctx.set_cursor(&Cursor::OpenHand);
                ctx.set_handled();
                ctx.request_paint();
            }
            Event::MouseDown(e) => {
                ctx.request_focus();

                self.is_mouse_down = true;
                if e.mods.alt() {
                    self.state = EditorState::Moving;
                    self.start_moving_pos = e.pos;
                    self.start_offset_x = self.offset_x;
                    self.start_offset_y = self.offset_y;
                } else if e.mods.shift() {
                    self.state = EditorState::BrushSelection;
                } else if e.mods.ctrl() && e.mods.shift() {
                    self.state = EditorState::ShapeSelection;
                    self.start_moving_pos = e.pos;
                } else {
                    self.state = EditorState::Drawing;

                    let transform = self.make_transform().inverse();
                    let p = transform * self.mouse_position;

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
            Event::MouseUp(_e) => {
                ctx.request_focus();

                self.is_mouse_down = false;

                match self.state {
                    EditorState::Drawing => {}
                    EditorState::Moving => {}
                    EditorState::ShapeSelection => {
                        let transform = self.make_transform().inverse();
                        let start = transform * self.start_moving_pos;
                        let end = transform * self.end_moving_pos;
                        let x1 = (start.x.min(end.x)) as u32;
                        let x2 = (start.x.max(end.x)) as u32;
                        let y1 = (start.y.min(end.y)) as u32;
                        let y2 = (start.y.max(end.y)) as u32;

                        let mut layer = data.layers[0].borrow_mut();
                        let mut v = layer.data.as_buffer_mut().unwrap().channel_mut(ChannelKind::Selection);
                        for y in y1..=y2 {
                            for x in x1..=x2 {
                                v.set(x, y, 255);
                            }
                        }
                    }
                    EditorState::BrushSelection => {
                        let mut layer = data.layers[0].borrow_mut();
                        let (mut sel, mut hot_sel) = layer.data.as_buffer_mut().unwrap().selection_mut();

                        for y in 0..sel.height() {
                            for x in 0..sel.width() {
                                sel.set(x, y, sel.get(x, y).saturating_add(hot_sel.get(x, y)));
                                hot_sel.set(x, y, 0);
                            }
                        }
                    }
                }
                self.state = EditorState::Drawing;
            }
            Event::KeyDown(e) => {
                ctx.request_paint();

                match e.code {
                    Code::BracketLeft => self.brush_size -= 1,
                    Code::BracketRight => self.brush_size += 1,
                    _ => ()
                }
            }
            Event::Wheel(e) => {
                match (e.mods.ctrl(), e.mods.alt(), e.mods.shift()) {
                    (true, false, false) => {
                        let new_scale = self.scale * e.wheel_delta.y.neg().signum().exp();

                        // From formula:
                        // (cursor_x - old_offset_x) / old_scale =
                        // (cursor_x - new_offset_x) / new_scale
                        // FIXME: cleanup
                        self.offset_x = -(self.mouse_position.x * new_scale - new_scale * self.offset_x - self.mouse_position.x * self.scale) / self.scale;
                        self.offset_y = -(self.mouse_position.y * new_scale - new_scale * self.offset_y - self.mouse_position.y * self.scale) / self.scale;
                        self.scale = new_scale;
                    }
                    (false, false, false) => {
                        self.offset_y += e.wheel_delta.y.neg();
                    }
                    (false, false, true) => {
                        self.offset_x += e.wheel_delta.y.neg();
                    }
                    _ => ()
                }
                ctx.set_handled();
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &AppData, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppData, _data: &AppData, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        bc.debug_check("Image");
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let transform = self.make_transform();

        let clip_rect = Rect::ZERO.with_size(ctx.size());
        ctx.clip(clip_rect);

        {
            let layer = data.layers[0].borrow();
            let r = layer.data.as_buffer().unwrap().channel(ChannelKind::Red).as_slice().unwrap();
            let g = layer.data.as_buffer().unwrap().channel(ChannelKind::Green).as_slice().unwrap();
            let b = layer.data.as_buffer().unwrap().channel(ChannelKind::Blue).as_slice().unwrap();
            let a = layer.data.as_buffer().unwrap().channel(ChannelKind::Alpha).as_slice().unwrap();
            let s = layer.data.as_buffer().unwrap().channel(ChannelKind::Selection);
            let hs = layer.data.as_buffer().unwrap().channel(ChannelKind::HotSelection);

            let mut overlay = layer.data.as_buffer().unwrap().channel(ChannelKind::Alpha).to_matrix();
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

            let alpha = overlay.as_slice();
            let rgba = &mut *layer.data.as_buffer().unwrap().interleaved.borrow_mut();
            match (data.channels[0].is_visible, data.channels[1].is_visible, data.channels[2].is_visible, data.channels[3].is_visible) {
                (true, false, false, false) => merge_channels(r, r, r, alpha, rgba),
                (false, true, false, false) => merge_channels(g, g, g, alpha, rgba),
                (false, false, true, false) => merge_channels(b, b, b, alpha, rgba),
                (false, false, false, true) => merge_channels(a, a, a, alpha, rgba),
                _ => merge_channels(r, g, b, alpha, rgba),
            }
        }

        data.layers[0].borrow().data.as_buffer().unwrap().to_piet(transform, ctx, self.interpolation);

        match self.state {
            EditorState::Drawing | EditorState::BrushSelection => {
                ctx.with_save(|ctx| {
                    let c = piet::Color::rgb8(90, 100, 20);
                    ctx.stroke(Circle::new(self.mouse_position, (self.brush_size as f64) / 2.0 * self.scale), &c, 1.0);
                });
            }
            EditorState::Moving => {}
            EditorState::ShapeSelection => {
                ctx.with_save(|ctx| {
                    let c = piet::Color::rgb8(0, 0, 0);
                    let mut ss = StrokeStyle::new();
                    ss.set_dash(vec![3.0, 1.0], 0.0);

                    ctx.stroke_styled(
                        Rect::from_points(self.start_moving_pos, self.end_moving_pos),
                        &c,
                        1.0,
                        &ss,
                    );
                });
            }
        }
    }
}

