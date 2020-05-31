use druid::{BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point, Rect, RenderContext, Size, UpdateCtx, Widget};
use piet::{ImageFormat, InterpolationMode};

use crate::AppData;
use crate::image_buffer::ImageBuffer;

pub struct Histogram {}

struct ChannelData {}


impl Widget<AppData> for Histogram {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut AppData, _env: &Env) {}

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
        fn make_image_data(image: &ImageBuffer, width: usize, height: usize) -> Vec<u8> {
            let mut result = vec![0; width * height * 4];
            let mut hist = [0u32; 256];

            for p in image.pixels[0].as_slice().iter() {
                hist[*p as usize] += 1;
            }

            let sum: usize = hist.iter().map(|it| *it as usize).max().unwrap();
            let mut normalized = [0u8; 256];
            for i in 0..256 {
                normalized[i] = (hist[i] as usize * 256 / sum) as u8;
            }

            for y in 0..height {
                for x in 0..width {
                    let c = if (255-normalized[x]) / 2 > y as u8 { 0 } else { 255 };

                    let ix = (y * width + x) * 4;
                    result[ix + 0] = c;
                    result[ix + 1] = c;
                    result[ix + 2] = c;
                    result[ix + 3] = c;
                }
            }
            result
        }

        // Let's burn some CPU to make a (partially transparent) image buffer
        let image_data = make_image_data(&data.image, 256, 128);
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
