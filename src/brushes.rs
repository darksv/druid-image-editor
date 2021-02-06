use crate::channels::ViewMut;

pub(crate) struct BasicBrush {
    size: u32,
    value: u8,
}

impl BasicBrush {
    pub(crate) fn new(size: u32, value: u8) -> Self {
        BasicBrush { size, value }
    }
}


pub(crate) trait Brush {
    fn apply(&self, image: ViewMut<'_, u8>, x: u32, y: u32);
}

impl Brush for BasicBrush {
    fn apply(&self, mut image: ViewMut<'_, u8>, x: u32, y: u32) {
        let brush_size = self.size as i32;

        let width = image.width() as i32;
        let height = image.height() as i32;

        let x0 = x as i32;
        let y0 = y as i32;

        for dy in -brush_size / 2..=brush_size / 2 {
            for dx in -brush_size / 2..=brush_size / 2 {
                let x = x0 + dx;
                let y = y0 + dy;

                if x < 0 || x >= width || y < 0 || y >= height {
                    continue;
                }

                let dist = ((x as f64 - x0 as f64).powf(2.0) + (y as f64 - y0 as f64).powf(2.0)).sqrt();

                if dist <= brush_size as f64 / 2.0 {
                    image.set(x as u32, y as u32, self.value);
                }
            }
        }
    }
}