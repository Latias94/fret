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
        EdgeRouteKind::Straight => super::segment::closest_point_on_segment(p, from, to).0,
        EdgeRouteKind::Step => super::step::closest_point_on_step_wire(p, from, to).0,
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
