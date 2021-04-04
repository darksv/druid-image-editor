use druid::{BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
            Point, Rect, RenderContext, Size, UpdateCtx, Widget, WidgetExt, WidgetPod};
use druid::widget::{Padding, Slider};

pub(crate) struct ColorPicker {
    red: WidgetPod<f64, Padding<f64, Slider>>,
    green: WidgetPod<f64, Padding<f64, Slider>>,
    blue: WidgetPod<f64, Padding<f64, Slider>>,
}

impl ColorPicker {
    pub(crate) fn new() -> Self {
        Self {
            red: WidgetPod::new(Slider::new().padding(5.0)),
            green: WidgetPod::new(Slider::new().padding(5.0)),
            blue: WidgetPod::new(Slider::new().padding(5.0)),
        }
    }
}

#[derive(Copy, Clone, Debug, Data, Eq, PartialEq)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
}

impl Color {
    pub(crate) fn new() -> Self {
        Self {
            r: 0,
            g: 0,
            b: 0,
        }
    }
}

fn u8_to_f64(x: u8) -> f64 {
    x as f64 / 255.0
}

fn f64_to_u8(x: f64) -> u8 {
    (x * 256.0) as u8
}

impl Widget<Color> for ColorPicker {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Color, env: &Env) {
        let mut r = u8_to_f64(data.r);
        let mut g = u8_to_f64(data.g);
        let mut b = u8_to_f64(data.b);

        self.red.event(ctx, event, &mut r, env);
        self.green.event(ctx, event, &mut g, env);
        self.blue.event(ctx, event, &mut b, env);

        data.r = f64_to_u8(r);
        data.g = f64_to_u8(g);
        data.b = f64_to_u8(b);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &Color, env: &Env) {
        self.red.lifecycle(ctx, event, &0.0, env);
        self.green.lifecycle(ctx, event, &0.0, env);
        self.blue.lifecycle(ctx, event, &0.0, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Color, data: &Color, env: &Env) {
        self.red.update(ctx, &u8_to_f64(data.r), env);
        self.green.update(ctx, &u8_to_f64(data.g), env);
        self.blue.update(ctx, &u8_to_f64(data.b), env);

        if old_data != data {
            ctx.request_paint();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Color, env: &Env) -> Size {
        let rect_height = 20.0;

        let bc = BoxConstraints::new(Size::new(bc.max().width, bc.min().height), bc.max());
        let red_size = self.red.layout(ctx, &bc, &0.0, env);
        let green_size = self.green.layout(ctx, &bc, &0.0, env);
        let blue_size = self.blue.layout(ctx, &bc, &0.0, env);
        self.red.set_origin(ctx, &1.0, env, Point::new(0.0, rect_height));
        self.green.set_origin(ctx, &1.0, env, Point::new(0.0, rect_height + red_size.height));
        self.blue.set_origin(ctx, &1.0, env, Point::new(0.0, rect_height + red_size.height + green_size.height));
        Size::new(
            red_size.width,
            red_size.height + green_size.height + blue_size.height + rect_height,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Color, env: &Env) {
        let r = data.r as f64 / 255.0;
        let g = data.g as f64 / 255.0;
        let b = data.b as f64 / 255.0;
        let rect = Rect::from_origin_size(Point::ORIGIN, (ctx.size().width, 20.0));
        ctx.fill(rect, &druid::Color::rgb(r, g, b));
        self.red.paint(ctx, &r, env);
        self.green.paint(ctx, &g, env);
        self.blue.paint(ctx, &b, env);
    }
}