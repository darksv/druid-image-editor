use std::cell::RefCell;
use std::sync::Arc;

use druid::{BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, RenderContext, Size, UpdateCtx, Widget};
use druid::widget::ListIter;

use crate::state::{Channel, Layer};

impl ListIter<Layer> for Arc<Vec<RefCell<Layer>>> {
    fn for_each(&self, mut cb: impl FnMut(&Layer, usize)) {
        for (index, item) in self.iter().enumerate() {
            let item = item.borrow();
            cb(&*item, index);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Layer, usize)) {
        for (index, item) in self.iter().enumerate() {
            let mut item = item.borrow_mut();
            cb(&mut *item, index);
        }
    }

    fn data_len(&self) -> usize {
        self.len()
    }
}

pub(crate) struct ChannelThumbnail;

impl Widget<Channel> for ChannelThumbnail {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Channel, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Channel, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Channel, _data: &Channel, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Channel, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Channel, _env: &Env) {
        let size = ctx.size();
        let rect = druid::Rect::from_origin_size(druid::Point::ORIGIN, size);
        ctx.fill(rect, &data.color);
        if data.is_selected {
            ctx.stroke(rect, &Color::rgba8(255, 255, 255, 255), 2.0);
        }
    }
}

pub(crate) struct LayerThumbnail;

impl Widget<Layer> for LayerThumbnail {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut Layer, _env: &Env) {}

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Layer, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &Layer, _data: &Layer, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &Layer, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Layer, _env: &Env) {
        let size = ctx.size();
        let rect = druid::Rect::from_origin_size(druid::Point::ORIGIN, size);
        // ctx.fill(rect, &data.color);
        if data.is_selected {
            ctx.stroke(rect, &Color::rgba8(255, 255, 255, 255), 2.0);
        }
    }
}