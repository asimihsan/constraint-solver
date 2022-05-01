use crate::primitives::{HorizontalSegment, VerticalSegment};
use crate::Unit;

/// Given a horizontal segment and a vertical segment, if they intersect return the intersection
/// point, else return None.
pub fn h_v_line_intersection(h: HorizontalSegment, v: VerticalSegment) -> Option<geo::Coordinate<Unit>> {
    let p0_x = h.0.start.x;
    let p0_y = h.0.start.y;
    let p1_x = h.0.end.x;
    let _p1_y = h.0.end.y;
    let p2_x = v.0.start.x;
    let p2_y = v.0.start.y;
    let _p3_x = v.0.end.x;
    let p3_y = v.0.end.y;

    #[allow(clippy::if_same_then_else)]
    if p0_x < p2_x && p1_x < p2_x {
        None
    } else if p0_x > p2_x && p1_x > p2_x {
        None
    } else if p2_y < p0_y && p3_y < p0_y {
        None
    } else if p2_y > p0_y && p3_y > p0_y {
        None
    } else {
        Some(geo::Coordinate::from((p2_x, p0_y)))
    }
}

// #[cfg(test)]
// proptest::proptest! {
//     #[test]
//     fn h_v_line_intersection_works() {
//
//     }
// }
