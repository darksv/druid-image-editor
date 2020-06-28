use druid::{Affine, BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Rect, RenderContext, Size, UpdateCtx, Widget, Point, Cursor};
use druid::KeyCode;
use piet::{InterpolationMode, StrokeStyle};
use std::ops::Neg;
use kurbo::{Circle, BezPath};
use std::iter;
use crate::AppData;
use crate::brushes::{BasicBrush, Brush};
use crate::contours::{find_contours, Contour};
use crate::image_buffer::merge_channels;
use crate::layers::View;

pub struct ImageEditor {
    interpolation: InterpolationMode,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
    mouse_position: Point,
    previous_mouse_position: Point,
    is_mouse_down: bool,
    state: EditorState,
    start_moving_pos: Point,
    start_offset_x: f64,
    start_offset_y: f64,
    end_moving_pos: Point,
    brush_size: u32,
    t: f64,
    is_contour_dirty: bool,
    contour: Vec<Contour>,
}

enum EditorState {
    Drawing,
    Moving,
    Selecting,
}

impl ImageEditor {
    pub fn new() -> Self {
        ImageEditor {
            interpolation: InterpolationMode::NearestNeighbor,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            mouse_position: Default::default(),
            previous_mouse_position: Default::default(),
            is_mouse_down: false,
            state: EditorState::Drawing,
            start_moving_pos: Default::default(),
            end_moving_pos: Default::default(),
            start_offset_x: 0.0,
            start_offset_y: 0.0,
            brush_size: 1,
            t: 0.0,
            is_contour_dirty: true,
            contour: vec![],
        }
    }

    fn make_transform(&self) -> Affine {
        Affine::new([self.scale, 0.0, 0.0, self.scale, self.offset_x, self.offset_y])
    }
}

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

fn interpolate_points(begin: Point, end: Point, mut f: impl FnMut(Point)) {
    let (begin, end) = if begin.x < end.x { (begin, end) } else { (end, begin) };

    let x0 = begin.x as i32;
    let y0 = begin.y as i32;
    let x1 = end.x as i32;
    let y1 = end.y as i32;

    bresenham::plot_line(x0, y0, x1, y1, |x, y| f(Point::new(x as f64, y as f64)));
}

fn gaussian(bytes: &[u8], width: usize, height: usize, out: &mut [u8]) {
    for y in 0..height {
        for x in 1..width - 1 {
            out[y * width + x] =
                bytes[y * width + x - 1] / 4
                    + bytes[y * width + x] / 4
                    + bytes[y * width + x + 1] / 4;
        }
    }
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

impl Widget<AppData> for ImageEditor {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::MouseMove(e) => {
                ctx.request_focus();

                self.previous_mouse_position = self.mouse_position;
                self.mouse_position = e.pos;

                if self.is_mouse_down {
                    match self.state {
                        EditorState::Drawing => {
                            let transform = self.make_transform().inverse();
                            let begin = transform * self.previous_mouse_position;
                            let end = transform * self.mouse_position;

                            interpolate_points(begin, end, |p| {
                                BasicBrush::new(self.brush_size).apply(
                                    data.image.pixels[0].as_view_mut(),
                                    p.x as u32,
                                    p.y as u32,
                                );
                            });

                            self.is_contour_dirty = true;
                        }
                        EditorState::Moving => {
                            let image_pos_x = self.start_moving_pos.x - self.start_offset_x;
                            let image_pos_y = self.start_moving_pos.y - self.start_offset_y;
                            self.offset_x = e.pos.x - image_pos_x;
                            self.offset_y = e.pos.y - image_pos_y;
                        }
                        EditorState::Selecting => {
                            self.end_moving_pos = self.mouse_position;
                        }
                    }
                }

                ctx.set_cursor(&Cursor::OpenHand);
                ctx.set_handled();
                ctx.request_paint();
            }
            Event::MouseDown(e) => {
                ctx.request_focus();

                self.is_mouse_down = true;

                let transform = self.make_transform().inverse();
                let p = transform * self.mouse_position;

                BasicBrush::new(self.brush_size).apply(
                    data.image.pixels[0].as_view_mut(),
                    p.x as u32,
                    p.y as u32,
                );


                // let w = data.image.width();
                // let offset_x = 10;
                // let offset_y = 10;
                // for y in offset_y..offset_y + 5 {
                //     for x in offset_x..offset_x + 5 {
                //         data.image.selection[(y * w + x) as usize] = 255;
                //         data.image.pixels[0][(y * w + x) as usize] = 255;
                //     }
                // }
                //
                // let w = data.image.width();
                // for y in offset_y + 3..offset_y + 8 {
                //     for x in offset_x + 3..offset_x + 8 {
                //         data.image.selection[(y * w + x) as usize] = 255;
                //         data.image.pixels[0][(y * w + x) as usize] = 255;
                //     }
                // }

                // let c = find_contours(
                //     View::new(
                //         &data.image.selection,
                //         0,
                //         0,
                //         data.image.width(),
                //         data.image.height(),
                //     ));
                // for c in c {
                //     for (x, p) in c.points.iter().enumerate() {
                //         data.image.pixels[1][(p.y * w + p.x) as usize] = (x * 255 / c.points.len()) as u8;
                //     }
                // }


                self.is_contour_dirty = true;

                if e.mods.alt {
                    self.state = EditorState::Moving;
                    self.start_moving_pos = e.pos;
                    self.start_offset_x = self.offset_x;
                    self.start_offset_y = self.offset_y;
                } else if e.mods.ctrl && e.mods.shift {
                    self.state = EditorState::Selecting;
                    self.start_moving_pos = e.pos;
                }
            }
            Event::MouseUp(_e) => {
                ctx.request_focus();

                self.is_mouse_down = false;
                self.state = EditorState::Drawing;
            }
            Event::KeyDown(e) => {
                dbg!(e);
                match e.key_code {
                    KeyCode::LeftBracket => self.brush_size -= 1,
                    KeyCode::RightBracket => self.brush_size += 1,
                    _ => ()
                }
            }
            Event::Wheel(e) => {
                match (e.mods.ctrl, e.mods.alt, e.mods.shift) {
                    (true, false, false) => {
                        let new_scale = self.scale * e.wheel_delta.y.neg().signum().exp();

                        // From formula:
                        // (cursor_x - old_offset_x) / old_scale =
                        // (cursor_x - new_offset_x) / new_scale
                        // FIXME: cleanup
                        self.offset_x = -(self.mouse_position.x * new_scale - new_scale * self.offset_x - self.mouse_position.x * self.scale) / self.scale;
                        self.offset_y = -(self.mouse_position.y * new_scale - new_scale * self.offset_y - self.mouse_position.y * self.scale) / self.scale;
                        self.scale = new_scale;
                    }
                    (false, false, false) => {
                        self.offset_y += e.wheel_delta.y.neg();
                    }
                    (false, false, true) => {
                        self.offset_x += e.wheel_delta.y.neg();
                    }
                    _ => ()
                }
                ctx.set_handled();
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, _data: &AppData, _env: &Env) {
        match event {
            LifeCycle::AnimFrame(interval) => {
                self.t += (*interval as f64) * 1e-8;
                ctx.request_anim_frame();
            }
            LifeCycle::WidgetAdded => {
                ctx.request_anim_frame();
            }
            _ => {}
        }
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
        let s = std::time::Instant::now();
        let transform = self.make_transform();

        let clip_rect = Rect::ZERO.with_size(ctx.size());
        ctx.clip(clip_rect);

        match (data.layers[0].is_visible, data.layers[1].is_visible, data.layers[2].is_visible) {
            (true, false, false) => {
                merge_channels(
                    data.image.pixels[0].as_slice(),
                    data.image.pixels[0].as_slice(),
                    data.image.pixels[0].as_slice(),
                    data.image.pixels[3].as_slice(),
                    &mut *data.image.interleaved.borrow_mut(),
                );
            }
            (false, true, false) => {
                merge_channels(
                    data.image.pixels[1].as_slice(),
                    data.image.pixels[1].as_slice(),
                    data.image.pixels[1].as_slice(),
                    data.image.pixels[3].as_slice(),
                    &mut *data.image.interleaved.borrow_mut(),
                );
            }
            (false, false, true) => {
                merge_channels(
                    data.image.pixels[2].as_slice(),
                    data.image.pixels[2].as_slice(),
                    data.image.pixels[2].as_slice(),
                    data.image.pixels[3].as_slice(),
                    &mut *data.image.interleaved.borrow_mut(),
                );
            }
            _ => {
                merge_channels(
                    data.image.pixels[0].as_slice(),
                    data.image.pixels[1].as_slice(),
                    data.image.pixels[2].as_slice(),
                    data.image.pixels[3].as_slice(),
                    &mut *data.image.interleaved.borrow_mut(),
                );
            }
        }

        data.image.to_piet(transform, ctx, self.interpolation);

        match self.state {
            EditorState::Drawing => {
                ctx.with_save(|ctx| {
                    let c = piet::Color::rgb8(90, 100, 20);
                    ctx.stroke(Circle::new(self.mouse_position, (self.brush_size as f64) / 2.0 * self.scale), &c, 1.0);
                });
            }
            EditorState::Moving => {}
            EditorState::Selecting => {
                ctx.with_save(|ctx| {
                    let c = piet::Color::rgb8(0, 0, 0);
                    let mut ss = StrokeStyle::new();
                    ss.set_dash(vec![3.0, 1.0], 0.0);

                    ctx.stroke_styled(
                        Rect::from_points(self.start_moving_pos, self.end_moving_pos),
                        &c,
                        1.0,
                        &ss,
                    );
                });
            }
        }

        // let x1 = Point::new(0.0, 0.0);
        // let x2 = Point::new(1.0, 1.0);
        // let pixel_size = transform * x2 - transform * x1;
        //
        // if pixel_size.x > 20.0 {
        //     ctx.with_save(|ctx| {
        //         dbg!();
        //         let mut path = BezPath::new();
        //         for y in 0..100 {
        //             // for x in 0..100 {
        //             path.line_to(transform.inverse() * Point::new(0.0, y as f64));
        //             path.move_to(transform.inverse() * Point::new(100.0, y as f64));
        //             // }
        //         }
        //
        //         dbg!(&path);
        //
        //         ctx.stroke(
        //             &path,
        //             &piet::Color::rgb8(100, 20, 100),
        //             1.0,
        //         );
        //     });
        // }


        ctx.with_save(|ctx| {
            // NOTE: we must not use ctx.transform to avoid scaling the contour line width

            let mut path = BezPath::new();
            let mut p2 = BezPath::new();

            if self.is_contour_dirty {
                self.contour = find_contours(
                    View::new(
                        data.image.selection.as_slice(),
                        0,
                        0,
                        data.image.width(),
                        data.image.height(),
                    ));
                self.is_contour_dirty = false;
            }

            for contour in self.contour.iter() {
                // path.move_to(transform * Point::new(contour.points[0].x as f64, contour.points[0].y as f64));
                //
                // for window in contour.points.windows(3) {
                //     match window {
                //
                //         // __|
                //         [p, c, n] if p.x == c.x + 1 && p.y == c.y - 1 && n.x == c.x - 1 && n.y == c.y => {
                //             path.line_to(transform * Point::new(p.x as f64, (p.y + 1) as f64));
                //         }
                //         // |-
                //         // |
                //         [p, c, n] if p.x == c.x + 1 && p.y == c.y && n.x == c.x && n.y == c.y + 1 => {
                //             path.line_to(transform * Point::new(c.x as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new(c.x as f64, (n.y + 1) as f64));
                //         }
                //         //   |
                //         // |
                //         //   |
                //         [p, c, n] if p.x == c.x + 1 && p.y == c.y - 1 && n.x == c.x + 1 && n.y == c.y + 1 => {
                //             path.line_to(transform * Point::new(c.x as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y + 1) as f64));
                //         }
                //         // |_
                //         [p, c, n] if p.x == c.x && p.y == c.y - 1 && n.x == c.x + 1 && n.y == c.y => {
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 2) as f64, (c.y + 1) as f64));
                //         }
                //         //
                //         [p, c, n] if c.x == p.x + 1 && c.x == n.x - 1 && c.y == p.y + 1 && c.y == n.y + 1 => {
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y) as f64));
                //         }
                //
                //         [p, c, n] if p.x == c.x - 1 && p.y == c.y && n.x == c.x && n.y == c.y - 1 => {
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y - 1) as f64));
                //         }
                //
                //         [p, c, n] if p.x == c.x - 1 && p.y == c.y + 1 && n.x == c.x - 1 && n.y == c.y - 1 => {
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y + 1) as f64));
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y) as f64));
                //         }
                //         [p, c, n] if p.x == c.x && p.y == c.y + 1 && n.x == c.x - 1 && n.y == c.y => {
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new((c.x - 1) as f64, (c.y) as f64));
                //         }
                //         [p, c, n] if p.x == c.x + 1 && p.y == c.y + 1 && n.x == c.x - 1 && n.y == c.y + 1 => {
                //             path.line_to(transform * Point::new((c.x + 1) as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y) as f64));
                //             path.line_to(transform * Point::new((c.x) as f64, (c.y + 1) as f64));
                //         }
                //         [p, c, n] if p.x == c.x && p.y == c.y - 1 && n.x == c.x && n.y == c.y + 1 => {
                //             path.line_to(transform * Point::new((p.x) as f64, (p.y) as f64));
                //             path.line_to(transform * Point::new((n.x) as f64, (n.y + 1) as f64));
                //         }
                //         [p, c, n] if p.x == c.x - 1 && p.y == c.y - 1 && n.x == c.x - 1 && n.y == c.y + 1 => {
                //             path.line_to(transform * Point::new((p.x) as f64, (p.y+1) as f64));
                //         }
                //         _ => {}
                //     }
                // }

                #[derive(PartialEq, Eq, Clone, Copy)]
                enum Dir { Left, Down, Right, Up }

                struct Tracker<'p> {
                    x: i32,
                    y: i32,
                    path: &'p mut BezPath,
                    transform: Affine,
                }

                impl<'p> Tracker<'p> {
                    fn new(path: &'p mut BezPath, transform: Affine, x: i32, y: i32) -> Self {
                        Tracker { x, y, path, transform }
                    }
                    fn mov(&mut self, dir: Dir) {
                        let curr_x = self.x;
                        let curr_y = self.y;

                        match dir {
                            Dir::Left => self.x -= 1,
                            Dir::Down => self.y += 1,
                            Dir::Right => self.x += 1,
                            Dir::Up => self.y -= 1,
                        };

                        self.path.move_to(self.transform * Point::new(curr_x as f64, curr_y as f64));
                        self.path.line_to(self.transform * Point::new(self.x as f64, self.y as f64));
                    }
                }

                match contour.points.as_slice() {
                    [] => (),
                    [point] => {
                        let mut t = Tracker::new(&mut path, transform, point.x as i32, point.y as i32);

                        t.mov(Dir::Down);
                        t.mov(Dir::Right);
                        t.mov(Dir::Up);
                        t.mov(Dir::Left);
                    }
                    [first, rest @ ..] => {
                        let mut tracker = Tracker::new(&mut path, transform, first.x as i32, first.y as i32);

                        //
                        //   6 5 4
                        //   7 x 3
                        //   8 1 2
                        //

                        let mut freeman = vec![];

                        for pair in contour.points.windows(2) {
                            if let [p1, p2] = pair {
                                let c = if p2.x == p1.x && p2.y == p1.y + 1 {
                                    1
                                } else if p2.x == p1.x + 1 && p2.y == p1.y + 1 {
                                    2
                                } else if p2.x == p1.x + 1 && p2.y == p1.y {
                                    3
                                } else if p2.x == p1.x + 1 && p2.y == p1.y - 1 {
                                    4
                                } else if p2.x == p1.x && p2.y == p1.y - 1 {
                                    5
                                } else if p2.x == p1.x - 1 && p2.y == p1.y - 1 {
                                    6
                                } else if p2.x == p1.x - 1 && p2.y == p1.y {
                                    7
                                } else if p2.x == p1.x - 1 && p2.y == p1.y + 1 {
                                    8
                                } else {
                                    unreachable!()
                                };

                                freeman.push(c);
                            }
                        }

                        println!("freeman = {:?}", freeman);
                        //
                        //
                        // let mut previous = first;
                        // let mut prev_dir = Dir::Down;
                        // for current in rest.iter() {
                        //     let (next, next2) = if previous.x == current.x && previous.y + 1 == current.y {
                        //         // 1
                        //         // 2
                        //         (Dir::Down, None)
                        //     } else if previous.x + 1 == current.x && previous.y == current.y {
                        //         // 1 2
                        //         (Dir::Right, None)
                        //     } else if previous.x == current.x && previous.y == current.y + 1 {
                        //         // 2
                        //         // 1
                        //         (Dir::Up, None)
                        //     } else if current.x == previous.x - 1 && current.y == previous.y {
                        //         // 2 1
                        //         (Dir::Left, None)
                        //     } else if current.x == previous.x + 1 && current.y == previous.y + 1 {
                        //         // 1
                        //         //   2
                        //         (Dir::Down, Some(Dir::Right))
                        //     } else {
                        //         dbg!(previous, current);
                        //
                        //         tracker.mov(prev_dir);
                        //         break;
                        //     };
                        //
                        //     // if next2.is_none() {
                        //     //     if prev_dir != next {
                        //     //         tracker.mov(prev_dir);
                        //     //     }
                        //     // }
                        //
                        //
                        //     tracker.mov(next);
                        //     prev_dir = next;
                        //
                        //     if let Some(next) = next2 {
                        //         tracker.mov(next);
                        //         prev_dir = next;
                        //     }
                        //
                        //
                        //     previous = current;
                        // }


                        // use std::cmp::Ordering;
                        // while idx + 1 < contour.points.len() {
                        //     match (
                        //         contour.points[idx + 1].x.cmp(&contour.points[idx].x),
                        //         contour.points[idx + 1].y.cmp(&contour.points[idx].y),
                        //     ) {
                        //         (Ordering::Equal, Ordering::Greater) => {
                        //             t.mov(Dir::Right);
                        //             while idx + 1 < contour.points.len() && contour.points[idx + 1].y == cy {
                        //                 idx += 1;
                        //                 cx += 1;
                        //                 t.mov(Dir::Right);
                        //             }
                        //         }
                        //         _ => {
                        //             t.mov(Dir::Up);
                        //             while idx + 1 < contour.points.len() && contour.points[idx + 1].x == cx {
                        //                 idx += 1;
                        //                 cy -= 1;
                        //                 t.mov(Dir::Up);
                        //             }
                        //
                        //             t.mov(Dir::Left);
                        //             while idx + 1 < contour.points.len() && contour.points[idx + 1].y == cy {
                        //                 idx += 1;
                        //                 cx -= 1;
                        //                 t.mov(Dir::Left);
                        //             }
                        //         }
                        //     }
                        //
                        //     t.mov(Dir::Down);
                        //     while idx + 1 < contour.points.len() && contour.points[idx + 1].x == cx {
                        //         idx += 1;
                        //         cy += 1;
                        //         t.mov(Dir::Down);
                        //     }
                        //
                        //
                        //
                        //
                        //
                        //
                        //     idx += 1;
                        // }
                        //
                        // let mut previous = first;
                        // for point in rest {
                        //     use std::cmp::Ordering;
                        //
                        //     match (point.x.cmp(&previous.x), point.y.cmp(&previous.y)) {
                        //         (Ordering::Equal, Ordering::Greater) => {
                        //             t.mov(Dir::Down);
                        //         }
                        //         (Ordering::Greater, Ordering::Equal) => {
                        //             t.mov(Dir::Right);
                        //         }
                        //         (Ordering::Equal, Ordering::Less) => {
                        //             t.mov(Dir::Up);
                        //         }
                        //         (Ordering::Less, Ordering::Equal) => {
                        //             t.mov(Dir::Left);
                        //         }
                        //         (Ordering::Greater, Ordering::Greater) => {
                        //             t.mov(Dir::Down);
                        //             t.mov(Dir::Right);
                        //         }
                        //         (Ordering::Less, Ordering::Less) => {
                        //             t.mov(Dir::Up);
                        //             t.mov(Dir::Left);
                        //         }
                        //         (Ordering::Less, Ordering::Greater) => {
                        //             t.mov(Dir::Down);
                        //             t.mov(Dir::Left);
                        //         }
                        //         (Ordering::Greater, Ordering::Less) => {
                        //             t.mov(Dir::Up);
                        //             t.mov(Dir::Right);
                        //         }
                        //         (Ordering::Equal, Ordering::Equal) => {
                        //
                        //             // t.mov(Dir::Down);
                        //         }
                        //     }
                        //
                        //     previous = point;
                        // }
                    }
                }
            }

            ctx.stroke(&p2, &piet::Color::rgb8(0, 0, 255), 1.0);

            const PRIMARY_COLOR: piet::Color = piet::Color::rgb8(0, 0, 0);
            const SECONDARY_COLOR: piet::Color = piet::Color::rgb8(255, 255, 255);

            let mut style = StrokeStyle::new();
            style.set_dash(vec![5.0, 2.5], self.t % 7.5);

            ctx.stroke_styled(
                &path,
                &PRIMARY_COLOR,
                1.0,
                &style,
            );

            let mut style = StrokeStyle::new();
            style.set_dash(vec![2.5, 5.0], 5.0 + self.t % 7.5);

            ctx.stroke_styled(
                &path,
                &SECONDARY_COLOR,
                1.0,
                &style,
            );
        });

        // println!("paint time = {:?}", s.elapsed());
    }
}

