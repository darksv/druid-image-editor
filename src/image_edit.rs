use druid::piet::InterpolationMode;
use druid::widget::Viewport;
use druid::{
    BoxConstraints, Code, Cursor, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget,
};

use crate::state::AppData;
use crate::tools::{BrushSelectionTool, DrawTool, MovingTool, ShapeSelectionTool, Tool, ToolRef};
use druid::scroll_component::ScrollComponent;

pub struct ImageEditor {
    interpolation: InterpolationMode,
    mouse_position: Point,
    previous_mouse_position: Point,
    is_mouse_down: bool,
    state: EditorState,
    shape_sel_tool: ShapeSelectionTool,
    moving_tool: MovingTool,
    scroll_component: ScrollComponent,
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
            shape_sel_tool: ShapeSelectionTool::new(),
            moving_tool: MovingTool::new(),
            scroll_component: ScrollComponent::new(),
        }
    }

    fn tool_mut(&mut self, data: &AppData) -> ToolRef {
        match self.state {
            EditorState::Drawing => ToolRef::Owned(Box::new(DrawTool::new(
                data.brush_size.round() as u32,
                [
                    data.brush_color.r,
                    data.brush_color.g,
                    data.brush_color.b,
                    255,
                ],
            ))),
            EditorState::Moving => ToolRef::Ref(&mut self.moving_tool),
            EditorState::ShapeSelection => ToolRef::Ref(&mut self.shape_sel_tool),
            EditorState::BrushSelection => ToolRef::Owned(Box::new(BrushSelectionTool::new(
                data.brush_size.round() as u32,
            ))),
        }
    }

    fn viewport(&self, data: &AppData, size: Size) -> Viewport {
        let (width, height) = data.layers[0].borrow().data.as_buffer().unwrap().size();
        let content_size = Size::new(width as f64, height as f64);

        Viewport {
            content_size,
            view_origin: Point::new(-self.moving_tool.offset_x, -self.moving_tool.offset_y),
            view_size: size,
        }
    }
}

impl Widget<AppData> for ImageEditor {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, env: &Env) {
        let mut port = self.viewport(data, ctx.size());
        self.scroll_component.event(&mut port, ctx, event, env);
        if ctx.is_handled() {
            self.moving_tool.offset_x = -port.view_origin.x;
            self.moving_tool.offset_y = -port.view_origin.y;
            return;
        }

        match event {
            Event::MouseMove(e) => {
                ctx.request_focus();

                self.previous_mouse_position = self.mouse_position;
                self.mouse_position = e.pos;

                if self.is_mouse_down {
                    let transform = self.moving_tool.transform();
                    let pos = self.mouse_position;
                    let prev_pos = self.previous_mouse_position;
                    self.tool_mut(data)
                        .as_mut()
                        .mouse_move(pos, prev_pos, transform, data);
                }

                ctx.set_cursor(&Cursor::Arrow);
                ctx.set_handled();
                ctx.request_paint();
            }
            Event::MouseDown(e) => {
                ctx.request_focus();

                self.is_mouse_down = true;
                self.state = if e.mods.alt() {
                    EditorState::Moving
                } else if e.mods.ctrl() && e.mods.shift() {
                    EditorState::ShapeSelection
                } else if e.mods.shift() {
                    EditorState::BrushSelection
                } else {
                    EditorState::Drawing
                };

                let transform = self.moving_tool.transform();
                let pos = self.mouse_position;
                self.tool_mut(data)
                    .as_mut()
                    .mouse_down(pos, transform, data);
            }
            Event::MouseUp(_e) => {
                ctx.request_focus();

                let transform = self.moving_tool.transform();
                self.tool_mut(data).as_mut().mouse_up(transform, data);

                self.state = EditorState::Drawing;
                self.is_mouse_down = false;
            }
            Event::KeyDown(e) => {
                ctx.request_paint();

                match e.code {
                    Code::BracketLeft => data.brush_size -= 1.0,
                    Code::BracketRight => data.brush_size += 1.0,
                    _ => (),
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

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &AppData, _data: &AppData, _env: &Env) {}

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        bc.debug_check("Image");
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, env: &Env) {
        let transform = self.moving_tool.transform();
        let clip_rect = Rect::ZERO.with_size(ctx.size());
        ctx.clip(clip_rect);
        data.ensure_fresh();
        data.layers[0].borrow().data.as_buffer().unwrap().to_piet(
            transform,
            ctx,
            self.interpolation,
        );

        let pos = self.mouse_position;
        let scale = self.moving_tool.scale();
        self.tool_mut(data).as_mut().overlay(ctx, pos, scale);

        self.scroll_component
            .draw_bars(ctx, &self.viewport(data, ctx.size()), env);
    }
}
