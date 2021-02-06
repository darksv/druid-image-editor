use druid::{BoxConstraints, Code, Cursor, Env, Event, EventCtx, LayoutCtx, LifeCycle,
            LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget, Color};
use druid::kurbo::Circle;
use druid::piet::{InterpolationMode, StrokeStyle};

use crate::AppData;
use crate::tools::{DrawTool, Tool, BrushSelectionTool, ShapeSelectionTool, MovingTool};

pub struct ImageEditor {
    interpolation: InterpolationMode,
    mouse_position: Point,
    previous_mouse_position: Point,
    is_mouse_down: bool,
    state: EditorState,
    brush_size: u32,
    shape_sel_tool: ShapeSelectionTool,
    moving_tool: MovingTool,
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
            mouse_position: Default::default(),
            previous_mouse_position: Default::default(),
            is_mouse_down: false,
            state: EditorState::Drawing,
            brush_size: 1,
            shape_sel_tool: ShapeSelectionTool::new(),
            moving_tool: MovingTool::new(),
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
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.moving_tool.transform(), data);
                        }
                        EditorState::Moving => {
                            self.moving_tool
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.moving_tool.transform(), data);
                        }
                        EditorState::ShapeSelection => {
                            self.shape_sel_tool
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.moving_tool.transform(), data);
                        }
                        EditorState::BrushSelection => {
                            BrushSelectionTool::new(self.brush_size)
                                .mouse_move(self.mouse_position, self.previous_mouse_position, self.moving_tool.transform(), data);
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
                    self.moving_tool
                        .mouse_down(e.pos, self.moving_tool.transform(), data);
                } else if e.mods.shift() {
                    self.state = EditorState::BrushSelection;
                } else if e.mods.ctrl() && e.mods.shift() {
                    self.state = EditorState::ShapeSelection;
                    self.shape_sel_tool
                        .mouse_down(e.pos, self.moving_tool.transform(), data);
                } else {
                    self.state = EditorState::Drawing;
                    DrawTool::new(self.brush_size)
                        .mouse_down(e.pos, self.moving_tool.transform(), data);
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
                            .mouse_up(self.moving_tool.transform(), data);
                    }
                    EditorState::BrushSelection => {
                        BrushSelectionTool::new(self.brush_size)
                            .mouse_up(self.moving_tool.transform(), data);
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
                self.moving_tool.wheel(e.pos, e.wheel_delta, e.mods);
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
        let transform = self.moving_tool.transform();

        let clip_rect = Rect::ZERO.with_size(ctx.size());
        ctx.clip(clip_rect);

        data.ensure_fresh();
        data.layers[0].borrow().data.as_buffer().unwrap().to_piet(transform, ctx, self.interpolation);

        match self.state {
            EditorState::Drawing | EditorState::BrushSelection => {
                ctx.with_save(|ctx| {
                    let c = Color::rgb8(90, 100, 20);
                    ctx.stroke(Circle::new(self.mouse_position, (self.brush_size as f64) / 2.0 * self.moving_tool.scale()), &c, 1.0);
                });
            }
            EditorState::Moving => {}
            EditorState::ShapeSelection => {
                ctx.with_save(|ctx| {
                    let c = Color::rgb8(0, 0, 0);
                    let mut ss = StrokeStyle::new();
                    ss.set_dash(vec![3.0, 1.0], 0.0);

                    ctx.stroke_styled(
                        Rect::from_points(self.shape_sel_tool.start_moving_pos.unwrap(), self.shape_sel_tool.end_moving_pos.unwrap()),
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

