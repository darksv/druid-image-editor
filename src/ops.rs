#[allow(unused)]
fn gaussian(bytes: &[u8], width: usize, height: usize, out: &mut [u8]) {
    for y in 0..height {
        for x in 1..width - 1 {
            out[y * width + x] = bytes[y * width + x - 1] / 4
                + bytes[y * width + x] / 4
                + bytes[y * width + x + 1] / 4;
        }
    }

    #[rustfmt::skip]
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
