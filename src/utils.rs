use druid::Point;

mod bresenham {
    // https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm#Algorithm_for_integer_arithmetic

    fn plot_line_low(x0: i32, y0: i32, x1: i32, y1: i32, mut f: impl FnMut(i32, i32)) {
        let dx = x1 - x0;
        let mut dy = y1 - y0;
        let mut yi = 1;
        if dy < 0 {
            yi = -1;
            dy = -dy;
        }
        let mut d = 2 * dy - dx;
        let mut y = y0;
        for x in x0..=x1 {
            f(x, y);
            if d > 0 {
                y += yi;
                d -= 2 * dx;
            }
            d += 2 * dy;
        }
    }

    fn plot_line_high(x0: i32, y0: i32, x1: i32, y1: i32, mut f: impl FnMut(i32, i32)) {
        let mut dx = x1 - x0;
        let dy = y1 - y0;
        let mut xi = 1;
        if dx < 0 {
            xi = -1;
            dx = -dx;
        }
        let mut d = 2 * dx - dy;
        let mut x = x0;
        for y in y0..=y1 {
            f(x, y);
            if d > 0 {
                x += xi;
                d -= 2 * dy;
            }
            d += 2 * dx;
        }
    }

    pub(crate) fn plot_line(x0: i32, y0: i32, x1: i32, y1: i32, f: impl FnMut(i32, i32)) {
        if (y1 - y0).abs() < (x1 - x0).abs() {
            if x0 > x1 {
                plot_line_low(x1, y1, x0, y0, f);
            } else {
                plot_line_low(x0, y0, x1, y1, f);
            }
        } else {
            if y0 > y1 {
                plot_line_high(x1, y1, x0, y0, f);
            } else {
                plot_line_high(x0, y0, x1, y1, f);
            }
        }
    }
}

pub(crate) fn interpolate_points(begin: Point, end: Point, mut f: impl FnMut(Point)) {
    let (begin, end) = if begin.x < end.x {
        (begin, end)
    } else {
        (end, begin)
    };

    let x0 = begin.x as i32;
    let y0 = begin.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    bresenham::plot_line(x0, y0, x1, y1, |x, y| f(Point::new(x as f64, y as f64)));
}
