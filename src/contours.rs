use crate::channels::{Matrix, View};

#[derive(Debug)]
pub struct Contour {
    pub points: Vec<Point<u32>>,
    previous: Option<u32>,
    next: Option<u32>,
    parent: Option<u32>,
    child: Option<u32>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    #[allow(unused)]
    const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum BorderType {
    Outer,
    Hole,
}

#[allow(unused)]
fn trace_border(
    start: Point<u32>,
    before_start: Point<u32>,
    mat: &mut Matrix<i8>,
    mut visit: impl FnMut(Point<u32>, bool, &mut Matrix<i8>),
) {
    let mut previous = before_start;
    let mut current = start;

    loop {
        // (3.3)
        let neighbours_counterclockwise = [
            Point::new(current.x + 1, current.y),
            Point::new(current.x + 1, current.y - 1),
            Point::new(current.x, current.y - 1),
            Point::new(current.x - 1, current.y - 1),
            Point::new(current.x - 1, current.y),
            Point::new(current.x - 1, current.y + 1),
            Point::new(current.x, current.y + 1),
            Point::new(current.x + 1, current.y + 1),
        ];

        let start_position = neighbours_counterclockwise
            .iter()
            .position(|x| x == &previous)
            .unwrap();

        let mut examined_with_zeros = [false; 8];
        let mut first_nonzero = None;
        for offset in 0..8 {
            // Visit every neighbour of the current pixel in a counterclockwise order
            // to find first pixel with nonzero value.
            // Meanwhile track all visited pixels.
            let idx = (offset + start_position + 1) % 8;
            let n = neighbours_counterclockwise[idx];
            if mat.get(n.x, n.y) == 0 {
                examined_with_zeros[idx] = true;
            } else {
                first_nonzero = Some(n);
                break;
            }
        }

        let was_pixel_on_right_visited = examined_with_zeros[0];
        visit(current, was_pixel_on_right_visited, mat);

        let next = first_nonzero.unwrap();
        // (3.5) - check whether contour has been closed
        if next == start && current == before_start {
            break;
        }

        // Move to the next point...
        previous = current;
        current = next;
    }
}

// pub(crate) fn find_contours2(
//     source: View<'_, u8>,
// ) -> Vec<Contour> {
//     // some kind of 1 pixel border is required to avoid out of area access
//     let mut mat = Matrix::new(source.width + 2, source.height + 2);
//     for y in 1..source.height() {
//         for x in 1..source.width() {
//             mat.set(x, y, (source.get(x, y) == 255) as i8);
//         }
//     }
//
//     let mut borders: HashMap<_, _> = HashMap::new();
//     borders.insert(1, BorderType::Hole);
//
//     let mut hierarchy: HashMap<_, _> = HashMap::new();
//     let mut contours = vec![];
//
//
//     let mut nbd = 1;
//     for i in 1..mat.height() - 1 {
//         let mut lnbd = 1;
//         for j in 1..mat.width() - 1 {
//             let ij = Point::new(j, i);
//
//             let prev = mat.get(j - 1, i);
//             let curr = mat.get(j, i);
//             let next = mat.get(j + 1, i);
//
//             let is_outer = curr == 1 && prev == 0;
//             let is_hole = curr >= 1 && next == 0;
//
//             if is_outer || is_hole {
//                 let (i2j2, border) = if is_outer {
//                     nbd += 1;
//                     (Point::new(j - 1, i), BorderType::Outer)
//                 } else {
//                     nbd += 1;
//                     if curr > 1 {
//                         lnbd = curr;
//                     }
//                     (Point::new(j + 1, i), BorderType::Hole)
//                 };
//
//                 // (2)
//                 let border_prime = borders[&lnbd];
//                 if border_prime == border {
//                     // B's parent is parent of B
// //                    println!("{} is sibling of {}", nbd, lnbd);
//                     hierarchy.insert(nbd, hierarchy[&lnbd]);
//                 } else {
//                     // B's is parent of B
// //                    println!("{} is parent of {}", lnbd, nbd);
//                     hierarchy.insert(nbd, lnbd);
//                 };
//
//                 // (3.1)
//                 let neighbours_clockwise = [
//                     Point::new(j + 1, i),
//                     Point::new(j + 1, i + 1),
//                     Point::new(j, i + 1),
//                     Point::new(j - 1, i + 1),
//                     Point::new(j - 1, i),
//                     Point::new(j - 1, i - 1),
//                     Point::new(j, i - 1),
//                     Point::new(j + 1, i - 1)
//                 ];
//
//                 let start = neighbours_clockwise.iter().position(|it| it == &i2j2).unwrap();
//                 let first_nonzero = (0..8).find_map(|offset| {
//                     let Point { x: j, y: i } = neighbours_clockwise[(offset + start) % 8];
//                     if mat.get(j, i) != 0 {
//                         Some(Point::new(j, i))
//                     } else {
//                         None
//                     }
//                 });
//
//                 let mut points = vec![];
//
//                 match first_nonzero {
//                     Some(i1j1) => {
//                         trace_border(ij, i1j1, &mut mat, |point, examined, mat| {
//                             points.push(Point::new(point.x - 1, point.y - 1));
//                             // (3.4)
//                             if examined && mat.get(point.x + 1, point.y) == 0 {
//                                 // (a)
//                                 mat.set(point.x, point.y, -nbd);
//                             } else if !examined && mat.get(point.x, point.y) == 1 {
//                                 // (b)
//                                 mat.set(point.x, point.y, nbd);
//                             }
//                         });
//                     }
//                     None => {
//                         // got single pixel
//                         points.push(Point::new(ij.x - 1 + 0, ij.y - 1 + 0));
//                         mat.set(j, i, -nbd);
//                     }
//                 }
//
//                 contours.push(Contour {
//                     points,
//                     previous: None,
//                     next: None,
//                     parent: None,
//                     child: None,
//                 });
//
//                 borders.insert(nbd, border);
//             }
//
//             // (4)
//             if curr != 0 && curr != 1 {
//                 lnbd = curr.abs();
//             }
//         }
//     }
//
// //    dbg!(hierarchy);
//
//     contours
// }

#[allow(unused)]
pub(crate) fn find_contours(source: View<'_, u8>) -> Vec<Contour> {
    let s = std::time::Instant::now();

    // some kind of 1 pixel border is required to avoid out of area access
    let mut mat = Matrix::new(source.width() + 2, source.height() + 2);
    for y in 1..source.height() {
        for x in 1..source.width() {
            mat.set(x, y, (source.get(x, y) == 255) as i8);
        }
    }
    dbg!(s.elapsed());

    let mut contours = vec![];

    for y in 1..mat.height() - 1 {
        for x in 1..mat.width() - 1 {
            let prev = mat.get(x - 1, y);
            let curr = mat.get(x, y);
            let next = mat.get(x + 1, y);

            let is_outer = curr == 1 && prev == 0;
            let is_hole = curr >= 1 && next == 0;

            if is_outer || is_hole {
                let i2j2 = if is_outer {
                    Point::new(x - 1, y)
                } else {
                    Point::new(x + 1, y)
                };

                // (3.1)
                let neighbours_clockwise = [
                    Point::new(x + 1, y),
                    Point::new(x + 1, y + 1),
                    Point::new(x, y + 1),
                    Point::new(x - 1, y + 1),
                    Point::new(x - 1, y),
                    Point::new(x - 1, y - 1),
                    Point::new(x, y - 1),
                    Point::new(x + 1, y - 1),
                ];

                let start = neighbours_clockwise
                    .iter()
                    .position(|it| it == &i2j2)
                    .unwrap();
                let first_nonzero = (0..8).find_map(|offset| {
                    let Point { x: j, y: i } = neighbours_clockwise[(offset + start) % 8];
                    if mat.get(j, i) != 0 {
                        Some(Point::new(j, i))
                    } else {
                        None
                    }
                });

                let mut points = vec![];

                match first_nonzero {
                    Some(i1j1) => {
                        trace_border(Point::new(x, y), i1j1, &mut mat, |point, examined, mat| {
                            points.push(Point::new(point.x - 1 + 1, point.y - 1 + 1));
                            // dbg!(point.x, point.y);
                            // (3.4)
                            if examined && mat.get(point.x + 1, point.y) == 0 {
                                // (a)
                                mat.set(point.x, point.y, -2);
                            } else if !examined && mat.get(point.x, point.y) == 1 {
                                // (b)
                                mat.set(point.x, point.y, 2);
                            }
                        });
                    }
                    None => {
                        // got single pixel
                        points.push(Point::new(x - 1 + 1, y - 1 + 1));
                        mat.set(x, y, -2);
                    }
                }

                contours.push(Contour {
                    points,
                    previous: None,
                    next: None,
                    parent: None,
                    child: None,
                });
            }
        }
    }
    dbg!(s.elapsed());
    contours
}

//
// #[cfg(test)]
// mod tests {
//     use crate::contours::{find_contours, Point, bounding_box};
//     use imageproc::rect::Rect;
//
//     #[test]
//     fn find_contours_example() {
//         let img = gray_image!(
//             255, 255, 255, 255, 255, 255, 255, 0,  0 ;
//             255,  0,   0 , 255,  0 ,  0 , 255, 0, 255;
//             255,  0,   0 , 255,  0 ,  0 , 255, 0,  0 ;
//             255, 255, 255, 255, 255, 255, 255, 0,  0
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 5);
//         assert_eq!(c[0].points, vec![Point { x: 0, y: 0 }, Point { x: 0, y: 3 }, Point { x: 8, y: 3 }, Point { x: 8, y: 0 }]);
//         assert_eq!(c[1].points, vec![Point { x: 0, y: 0 }, Point { x: 0, y: 1 }, Point { x: 0, y: 2 }, Point { x: 0, y: 3 }, Point { x: 1, y: 3 }, Point { x: 2, y: 3 }, Point { x: 3, y: 3 }, Point { x: 4, y: 3 }, Point { x: 5, y: 3 }, Point { x: 6, y: 3 }, Point { x: 6, y: 2 }, Point { x: 6, y: 1 }, Point { x: 6, y: 0 }, Point { x: 5, y: 0 }, Point { x: 4, y: 0 }, Point { x: 3, y: 0 }, Point { x: 2, y: 0 }, Point { x: 1, y: 0 }]);
//         assert_eq!(c[2].points, vec![Point { x: 0, y: 1 }, Point { x: 1, y: 0 }, Point { x: 2, y: 0 }, Point { x: 3, y: 1 }, Point { x: 3, y: 2 }, Point { x: 2, y: 3 }, Point { x: 1, y: 3 }, Point { x: 0, y: 2 }]);
//         assert_eq!(c[3].points, vec![Point { x: 3, y: 1 }, Point { x: 4, y: 0 }, Point { x: 5, y: 0 }, Point { x: 6, y: 1 }, Point { x: 6, y: 2 }, Point { x: 5, y: 3 }, Point { x: 4, y: 3 }, Point { x: 3, y: 2 }]);
//         assert_eq!(c[4].points, vec![Point { x: 8, y: 1 }]);
//     }
//
//     #[test]
//     fn find_contours_all_white() {
//         let img = gray_image!(
//             255, 255, 255, 255;
//             255, 255, 255, 255;
//             255, 255, 255, 255;
//             255, 255, 255, 255
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 2);
//         assert_eq!(c[0].points, vec![Point { x: 0, y: 0 }, Point { x: 0, y: 3 }, Point { x: 3, y: 3 }, Point { x: 3, y: 0 }]);
//         assert_eq!(c[1].points, vec![Point { x: 0, y: 0 }, Point { x: 0, y: 1 }, Point { x: 0, y: 2 }, Point { x: 0, y: 3 }, Point { x: 1, y: 3 }, Point { x: 2, y: 3 }, Point { x: 3, y: 3 }, Point { x: 3, y: 2 }, Point { x: 3, y: 1 }, Point { x: 3, y: 0 }, Point { x: 2, y: 0 }, Point { x: 1, y: 0 }]);
//     }
//
//     #[test]
//     fn find_contours_all_black() {
//         let img = gray_image!(
//             0, 0, 0, 0;
//             0, 0, 0, 0;
//             0, 0, 0, 0;
//             0, 0, 0, 0
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 0);
//     }
//
//     #[test]
//     fn find_contours_single_dot() {
//         let img = gray_image!(
//             0,  0 , 0;
//             0, 255, 0;
//             0,  0 , 0
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 2);
//         assert_eq!(c[0].points, vec![Point { x: 1, y: 1 }]);
//         assert_eq!(c[1].points, vec![Point { x: 1, y: 1 }]);
//     }
//
//     #[test]
//     fn find_contours_cross() {
//         let img = gray_image!(
//             0,  0 ,  0 ,  0 , 0;
//             0,  0 , 255,  0 , 0;
//             0, 255, 255, 255, 0;
//             0,  0 , 255,  0 , 0;
//             0,  0 ,  0 ,  0 , 0
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 2);
//         assert_eq!(c[0].points, vec![Point { x: 1, y: 1 }, Point { x: 1, y: 3 }, Point { x: 3, y: 3 }, Point { x: 3, y: 1 }]);
//         assert_eq!(c[1].points, vec![Point { x: 2, y: 1 }, Point { x: 1, y: 2 }, Point { x: 2, y: 3 }, Point { x: 3, y: 2 }]);
//     }
//
//     #[test]
//     fn find_contours_empty_square() {
//         let img = gray_image!(
//             0,  0 ,  0 ,  0 , 0;
//             0, 255, 255, 255, 0;
//             0, 255,  0 , 255, 0;
//             0, 255, 255, 255, 0;
//             0,  0 ,  0 ,  0 , 0
//         );
//         let c = find_contours(&img, 0, 0);
//         assert_eq!(c.len(), 3);
//         assert_eq!(c[0].points, vec![Point { x: 1, y: 1 }, Point { x: 1, y: 3 }, Point { x: 3, y: 3 }, Point { x: 3, y: 1 }]);
//         assert_eq!(c[1].points, vec![Point { x: 1, y: 1 }, Point { x: 1, y: 2 }, Point { x: 1, y: 3 }, Point { x: 2, y: 3 }, Point { x: 3, y: 3 }, Point { x: 3, y: 2 }, Point { x: 3, y: 1 }, Point { x: 2, y: 1 }]);
//         assert_eq!(c[2].points, vec![Point { x: 1, y: 2 }, Point { x: 2, y: 1 }, Point { x: 3, y: 2 }, Point { x: 2, y: 3 }]);
//     }
//
//     #[test]
//     fn bbox_none() {
//         let img = gray_image!(
//             0, 0, 0, 0, 0;
//             0, 0, 0, 0, 0;
//             0, 0, 0, 0, 0;
//             0, 0, 0, 0, 0;
//             0, 0, 0, 0, 0
//         );
//         let b = bounding_box(&img);
//         assert_eq!(b, None);
//     }
//
//     #[test]
//     fn bbox() {
//         let img = gray_image!(
//             0,  0 ,  0 ,  0 , 0;
//             0, 255, 255, 255, 0;
//             0, 255,  0 , 255, 0;
//             0, 255, 255, 255, 0;
//             0,  0 ,  0 ,  0 , 0
//         );
//         let b = bounding_box(&img);
//         assert_eq!(b, Some(Rect::at(1, 1).of_size(3, 3)));
//     }
//
//     #[test]
//     fn bbox2() {
//         let img = gray_image!(
//             255, 255, 255;
//             255,  0 , 255;
//             255, 255, 255
//         );
//         let b = bounding_box(&img);
//         assert_eq!(b, Some(Rect::at(0, 0).of_size(3, 3)));
//     }
//
//     #[test]
//     fn bbox_ex() {
//         let img = gray_image!(
//             255, 255, 255, 255, 255, 255, 255, 0,  0 ;
//             255,  0,   0 , 255,  0 ,  0 , 255, 0, 255;
//             255,  0,   0 , 255,  0 ,  0 , 255, 0,  0 ;
//             255, 255, 255, 255, 255, 255, 255, 0,  0
//         );
//         let b = bounding_box(&img);
//         assert_eq!(b, Some(Rect::at(0, 0).of_size(9, 4)));
//     }
// }
//
