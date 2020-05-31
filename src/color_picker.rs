use druid::{Widget, EventCtx, LifeCycle, PaintCtx, BoxConstraints, LifeCycleCtx, LayoutCtx, Event, Env, UpdateCtx, WidgetPod};
use kurbo::Size;
use druid::widget::{TextBox, Stepper};
use std::borrow::BorrowMut;

struct ColorPicker {
    red: WidgetPod<f64, Stepper>,
    green: WidgetPod<f64, Stepper>,
    blue: WidgetPod<f64, Stepper>,
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Widget<Color> for ColorPicker {
    fn event(&mut self, ctx: &mut EventCtx<'_>, event: &Event, data: &mut Color, env: &Env) {
        // Widget::event(self.red.borrow_mut(), ctx, event, &mut data.r, env);
        // Widget::event(self.red.borrow_mut(), ctx, event, &mut data.r, env);
        // Widget::event(self.red.borrow_mut(), ctx, event, &mut data.r, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx<'_>, event: &LifeCycle, data: &Color, env: &Env) {
        // self.red.borrow_mut().lifecycle(ctx, event, &data.r, env);
        // self.green.borrow_mut().lifecycle(ctx, event, &data.g, env);
        // self.blue.borrow_mut().lifecycle(ctx, event, &data.b, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, old_data: &Color, data: &Color, env: &Env) {
        // Widget::update(self.red.borrow_mut(), ctx, &old_data.r, &data.r, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx<'_, '_>, bc: &BoxConstraints, data: &Color, env: &Env) -> Size {
        todo!()
        // Widget::layout(self.red.borrow_mut(), ctx, bc, &data.r, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx<'_, '_>, data: &Color, env: &Env) {
        // self.red.borrow_mut().paint(ctx, &data.r, env);
    }
}