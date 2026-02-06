use super::super::*;

pub(super) fn closest_point_on_step_wire(p: Point, from: Point, to: Point) -> (Point, f32) {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let c0 = super::segment::closest_point_on_segment(p, from, p1);
    let c1 = super::segment::closest_point_on_segment(p, p1, p2);
    let c2 = super::segment::closest_point_on_segment(p, p2, to);
    [c0, c1, c2]
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((from, f32::INFINITY))
}

pub(super) fn step_wire_distance2(p: Point, from: Point, to: Point) -> f32 {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let d0 = super::segment::dist2_point_to_segment(p, from, p1);
    let d1 = super::segment::dist2_point_to_segment(p, p1, p2);
    let d2 = super::segment::dist2_point_to_segment(p, p2, to);
    d0.min(d1).min(d2)
}
