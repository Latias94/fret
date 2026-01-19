use super::*;

pub(super) fn wire_distance2(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
) -> f32 {
    fret_canvas::wires::bezier_wire_distance2(p, from, to, zoom, bezier_steps)
}

pub(super) fn closest_point_on_edge_route(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
    p: Point,
) -> Point {
    match route {
        EdgeRouteKind::Bezier => closest_point_on_wire_bezier(p, from, to, zoom, bezier_steps),
        EdgeRouteKind::Straight => closest_point_on_segment(p, from, to).0,
        EdgeRouteKind::Step => closest_point_on_step_wire(p, from, to).0,
    }
}

fn closest_point_on_wire_bezier(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
) -> Point {
    fret_canvas::wires::closest_point_on_bezier_wire(p, from, to, zoom, bezier_steps)
}

fn closest_point_on_step_wire(p: Point, from: Point, to: Point) -> (Point, f32) {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let c0 = closest_point_on_segment(p, from, p1);
    let c1 = closest_point_on_segment(p, p1, p2);
    let c2 = closest_point_on_segment(p, p2, to);
    [c0, c1, c2]
        .into_iter()
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or((from, f32::INFINITY))
}

fn closest_point_on_segment(p: Point, a: Point, b: Point) -> (Point, f32) {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        let d2 = apx * apx + apy * apy;
        return (a, d2);
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    (Point::new(Px(cx), Px(cy)), dx * dx + dy * dy)
}

pub(super) fn step_wire_distance2(p: Point, from: Point, to: Point) -> f32 {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let p1 = Point::new(Px(mx), from.y);
    let p2 = Point::new(Px(mx), to.y);
    let d0 = dist2_point_to_segment(p, from, p1);
    let d1 = dist2_point_to_segment(p, p1, p2);
    let d2 = dist2_point_to_segment(p, p2, to);
    d0.min(d1).min(d2)
}

pub(super) fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    let apx = p.x.0 - a.x.0;
    let apy = p.y.0 - a.y.0;
    let abx = b.x.0 - a.x.0;
    let aby = b.y.0 - a.y.0;

    let ab2 = abx * abx + aby * aby;
    if ab2 <= 1.0e-9 {
        return apx * apx + apy * apy;
    }

    let t = ((apx * abx + apy * aby) / ab2).clamp(0.0, 1.0);
    let cx = a.x.0 + t * abx;
    let cy = a.y.0 + t * aby;
    let dx = p.x.0 - cx;
    let dy = p.y.0 - cy;
    dx * dx + dy * dy
}
