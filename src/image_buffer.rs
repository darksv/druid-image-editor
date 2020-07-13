use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::path::Path;

use druid::{Affine, Data, PaintCtx, RenderContext, Size};
use piet::{ImageFormat, InterpolationMode};

use crate::ChannelKind;
use crate::channels::{Matrix, View, ViewMut};

/// Stored Image data.
#[derive(Clone, Data)]
pub struct ImageBuffer {
    #[data(ignore)]
    pixels: [Matrix<u8>; 4],
    #[data(ignore)]
    selection: Matrix<u8>,
    #[data(ignore)]
    hot_selection: Matrix<u8>,
    #[data(ignore)]
    pub(crate) interleaved: RefCell<Vec<u8>>,
    width: u32,
    height: u32,
    #[data(ignore)]
    format: ImageFormat,
}

impl ImageBuffer {
    pub(crate) fn channel(&self, kind: ChannelKind) -> View<'_, u8> {
        match kind {
            ChannelKind::Red => self.pixels[0].as_view(),
            ChannelKind::Green => self.pixels[1].as_view(),
            ChannelKind::Blue => self.pixels[2].as_view(),
            ChannelKind::Alpha => self.pixels[3].as_view(),
            ChannelKind::Selection => self.selection.as_view(),
            ChannelKind::HotSelection => self.hot_selection.as_view(),
        }
    }

    pub(crate) fn channel_mut(&mut self, kind: ChannelKind) -> ViewMut<'_, u8> {
        match kind {
            ChannelKind::Red => self.pixels[0].as_view_mut(),
            ChannelKind::Green => self.pixels[1].as_view_mut(),
            ChannelKind::Blue => self.pixels[2].as_view_mut(),
            ChannelKind::Alpha => self.pixels[3].as_view_mut(),
            ChannelKind::Selection => self.selection.as_view_mut(),
            ChannelKind::HotSelection => self.hot_selection.as_view_mut(),
        }
    }
    
    pub(crate) fn selection_mut(&mut self) -> (ViewMut<'_, u8>, ViewMut<'_, u8>) {
        (self.selection.as_view_mut(), self.hot_selection.as_view_mut())
    }

    pub(crate) fn width(&self) -> u32 { self.width }
    pub(crate) fn height(&self) -> u32 { self.height }
}

impl fmt::Debug for ImageBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImageData")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl ImageBuffer {
    /// Load an image from a DynamicImage from the image crate
    pub fn from_dynamic_image(image_data: image::DynamicImage) -> ImageBuffer {
        Self::from_dynamic_image_with_alpha(image_data)
    }

    /// Load an image from a DynamicImage with alpha
    pub fn from_dynamic_image_with_alpha(image_data: image::DynamicImage) -> ImageBuffer {
        let rgba_image = image_data.to_rgba();
        let (width, height) = rgba_image.dimensions();

        let mut r = Matrix::new(width, height);
        let mut g = Matrix::new(width, height);
        let mut b = Matrix::new(width, height);
        let mut a = Matrix::new(width, height);

        let size_in_bytes = width as usize * height as usize * 4;

        let m = unsafe { std::slice::from_raw_parts(rgba_image.as_ptr(), size_in_bytes) };
        for (i, pix) in m.chunks_exact(4).enumerate() {
            r.as_slice_mut()[i] = pix[0];
            g.as_slice_mut()[i] = pix[1];
            b.as_slice_mut()[i] = pix[2];
            a.as_slice_mut()[i] = pix[3];
        }

        ImageBuffer {
            interleaved: RefCell::new(vec![0; size_in_bytes]),
            pixels: [r, g, b, a],
            selection: Matrix::new(width, height),
            hot_selection: Matrix::new(width, height),
            width,
            height,
            format: ImageFormat::RgbaSeparate,
        }
    }

    /// Attempt to load an image from the file at the provided path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let image_data = image::open(path).map_err(|e| e)?;
        Ok(ImageBuffer::from_dynamic_image(image_data))
    }

    /// Get the size in pixels of the contained image.
    fn get_size(&self) -> Size {
        Size::new(self.width as f64, self.height as f64)
    }

    /// Convert ImageData into Piet draw instructions
    pub(crate) fn to_piet(&self, offset_matrix: Affine, ctx: &mut PaintCtx, interpolation: InterpolationMode) {
        ctx.with_save(|ctx| {
            let size = self.get_size();
            // Background around the image
            ctx.fill(size.to_rect(), &piet::Color::rgb8(38, 38, 38));

            ctx.transform(offset_matrix);
            let im = ctx
                .make_image(
                    size.width as usize,
                    size.height as usize,
                    &*self.interleaved.borrow(),
                    // FIXME: hardcoded format... should be `self.format`
                    ImageFormat::RgbaPremul,
                )
                .unwrap();
            ctx.draw_image(&im, size.to_rect(), interpolation);
        })
    }
}

#[inline(never)]
pub fn merge_channels(r: &[u8], g: &[u8], b: &[u8], a: &[u8], rgba: &mut [u8]) {
    assert_eq!(r.len(), g.len());
    assert_eq!(g.len(), b.len());
    assert_eq!(b.len(), a.len());
    assert_eq!(r.len() * 4, rgba.len());

    #[target_feature(enable = "avx2")]
    #[target_feature(enable = "avx")]
    #[inline]
    unsafe fn merge_avx2(r: &[u8], g: &[u8], b: &[u8], a: &[u8], rgba: &mut [u8]) {
        use std::arch::x86_64 as x86;

        let mut out_idx = 0;
        for i in (0..r.len()).step_by(32) {
            let vr = x86::_mm256_loadu_si256(r[i..].as_ptr().cast());
            let vg = x86::_mm256_loadu_si256(g[i..].as_ptr().cast());
            let vb = x86::_mm256_loadu_si256(b[i..].as_ptr().cast());
            let va = x86::_mm256_loadu_si256(a[i..].as_ptr().cast());

            let vrg_lo = x86::_mm256_unpacklo_epi8(vr, vg);
            let vba_lo = x86::_mm256_unpacklo_epi8(vb, va);
            let vrgba_lo_lo = x86::_mm256_unpacklo_epi16(vrg_lo, vba_lo);
            let vrgba_lo_hi = x86::_mm256_unpackhi_epi16(vrg_lo, vba_lo);

            let vrg_hi = x86::_mm256_unpackhi_epi8(vr, vg);
            let vba_hi = x86::_mm256_unpackhi_epi8(vb, va);
            let vrgba_hi_lo = x86::_mm256_unpacklo_epi16(vrg_hi, vba_hi);
            let vrgba_hi_hi = x86::_mm256_unpackhi_epi16(vrg_hi, vba_hi);

            let part_a = x86::_mm256_permute2x128_si256(vrgba_lo_lo, vrgba_lo_hi, 0b00_10_00_00);
            x86::_mm256_storeu_si256(rgba[out_idx..].as_mut_ptr().cast(), part_a);
            out_idx += 32;

            let part_b = x86::_mm256_permute2x128_si256(vrgba_hi_lo, vrgba_hi_hi, 0b00_10_00_00);
            x86::_mm256_storeu_si256(rgba[out_idx..].as_mut_ptr().cast(), part_b);
            out_idx += 32;

            let part_c = x86::_mm256_permute2x128_si256(vrgba_lo_lo, vrgba_lo_hi, 0b00_11_00_01);
            x86::_mm256_storeu_si256(rgba[out_idx..].as_mut_ptr().cast(), part_c);
            out_idx += 32;

            let part_d = x86::_mm256_permute2x128_si256(vrgba_hi_lo, vrgba_hi_hi, 0b00_11_00_01);
            x86::_mm256_storeu_si256(rgba[out_idx..].as_mut_ptr().cast(), part_d);
            out_idx += 32;
        }
    }

    #[inline]
    fn merge_scalar(r: &[u8], g: &[u8], b: &[u8], a: &[u8], rgba: &mut [u8]) {
        for i in 0..r.len() {
            rgba[i * 4 + 0] = r[i];
            rgba[i * 4 + 1] = g[i];
            rgba[i * 4 + 2] = b[i];
            rgba[i * 4 + 3] = a[i];
        }
    }

    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("avx") {
        unsafe { merge_avx2(r, g, b, a, rgba); }
    } else {
        merge_scalar(r, g, b, a, rgba);
    }
}
