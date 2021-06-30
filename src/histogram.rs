use druid::piet::{ImageFormat, InterpolationMode};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    Rect, RenderContext, Size, UpdateCtx, Widget,
};

use crate::image_buffer::ImageBuffer;
use crate::state::{AppData, ChannelKind};

pub struct Histogram {}

impl Widget<AppData> for Histogram {
    fn event(&mut self, _ctx: &mut EventCtx, _event: &Event, _data: &mut AppData, _env: &Env) {}

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
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        bc.debug_check("Image");
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        #[rustfmt::skip]
        fn make_image_data(image: &ImageBuffer, width: usize, height: usize) -> Vec<u8> {
            let mut normalized = [[0u8; 256]; 3];

            for channel in [ChannelKind::Red, ChannelKind::Green, ChannelKind::Blue] {
                let mut histogram = [0u32; 256];
                for value in image.channel(channel).as_slice().unwrap().iter().copied() {
                    histogram[value as usize] += 1;
                }

                let max_count: usize = histogram.iter().map(|it| *it as usize).max().unwrap();
                #[allow(clippy::needless_range_loop)]
                for value in 0..256 {
                    normalized[channel as usize][value] =
                        (histogram[value] as usize * 256 / max_count) as u8;
                }
            }

            let mut result = vec![0; width * height * 4];
            for y in 0..height {
                for x in 0..width {
                    let ix = (y * width + x) * 4;
                    result[ix + 0] = if (255 - normalized[0][x]) / 2 > y as u8 { 0 } else { 255 };
                    result[ix + 1] = if (255 - normalized[1][x]) / 2 > y as u8 { 0 } else { 255 };
                    result[ix + 2] = if (255 - normalized[2][x]) / 2 > y as u8 { 0 } else { 255 };
                    result[ix + 3] = 255;
                }
            }
            result
        }

        let image_data =
            make_image_data(data.layers[0].borrow().data.as_buffer().unwrap(), 256, 128);
        let image = ctx
            .make_image(256, 128, &image_data, ImageFormat::RgbaSeparate)
            .unwrap();

        let size = ctx.size();
        ctx.draw_image(
            &image,
            Rect::from_origin_size(Point::ORIGIN, size),
            InterpolationMode::Bilinear,
        );
    }
}
