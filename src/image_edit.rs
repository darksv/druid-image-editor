use std::ops::Neg;

use druid::{Affine, BoxConstraints, Code, Cursor, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget, Color};
use druid::kurbo::Circle;
use druid::piet::{InterpolationMode, StrokeStyle};

use crate::AppData;
use crate::tools::{DrawTool, Tool, BrushSelectionTool, ShapeSelectionTool};

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
    shape_sel_tool: ShapeSelectionTool,
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
            shape_sel_tool: ShapeSelectionTool::new(),
        }
    }

    fn make_transform(&self) -> Affine {
        Affine::new([self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y])
    }
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
                            DrawTool::new(self.brush_size)
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.make_transform(), data);
                        }
                        EditorState::Moving => {
                            let image_pos_x = self.start_moving_pos.x - self.start_offset_x;
                            let image_pos_y = self.start_moving_pos.y - self.start_offset_y;
                            self.offset_x = e.pos.x - image_pos_x;
                            self.offset_y = e.pos.y - image_pos_y;
                        }
                        EditorState::ShapeSelection => {
                            self.shape_sel_tool
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.make_transform(), data);
                        }
                        EditorState::BrushSelection => {
                            BrushSelectionTool::new(self.brush_size)
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.make_transform(), data);
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
                    self.shape_sel_tool.mouse_down(e.pos, self.make_transform(), data);
                } else {
                    self.state = EditorState::Drawing;
                    DrawTool::new(self.brush_size)
                        .mouse_down(e.pos, self.make_transform(), data);
                }
            }
            Event::MouseUp(_e) => {
                ctx.request_focus();

                self.is_mouse_down = false;

                match self.state {
                    EditorState::Drawing => {}
                    EditorState::Moving => {}
                    EditorState::ShapeSelection => {
                        self.shape_sel_tool
                            .mouse_up(self.make_transform(), data);
                    }
                    EditorState::BrushSelection => {
                        BrushSelectionTool::new(self.brush_size)
                            .mouse_up(self.make_transform(), data);
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
        let s = std::time::Instant::now();
        let transform = self.make_transform();

        let clip_rect = Rect::ZERO.with_size(ctx.size());
        ctx.clip(clip_rect);

        data.ensure_fresh();
        data.layers[0].borrow().data.as_buffer().unwrap().to_piet(transform, ctx, self.interpolation);

        match self.state {
            EditorState::Drawing | EditorState::BrushSelection => {
                ctx.with_save(|ctx| {
                    let c = Color::rgb8(90, 100, 20);
                    ctx.stroke(Circle::new(self.mouse_position, (self.brush_size as f64) / 2.0 * self.scale), &c, 1.0);
                });
            }
            EditorState::Moving => {}
            EditorState::ShapeSelection => {
                ctx.with_save(|ctx| {
                    let c = Color::rgb8(0, 0, 0);
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

        dbg!(s.elapsed());
    }
}

