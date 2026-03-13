use super::*;

mod path;
mod route;
mod segment;
mod step;
#[cfg(test)]
mod tests;

pub(super) fn wire_distance2(
    p: Point,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
) -> f32 {
    route::wire_distance2(p, from, to, zoom, bezier_steps)
}

pub(super) fn closest_point_on_edge_route(
    route_kind: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    bezier_steps: usize,
    p: Point,
) -> Point {
    route::closest_point_on_edge_route(route_kind, from, to, zoom, bezier_steps, p)
}

pub(super) fn wire_distance2_path(
    p: Point,
    commands: &[fret_core::PathCommand],
    bezier_steps: usize,
) -> f32 {
    path::wire_distance2_path(p, commands, bezier_steps)
}

pub(super) fn closest_point_on_path(
    commands: &[fret_core::PathCommand],
    bezier_steps: usize,
    p: Point,
) -> Point {
    path::closest_point_on_path(commands, bezier_steps, p)
}

pub(super) fn path_start_end_tangents(
    commands: &[fret_core::PathCommand],
) -> Option<(Point, Point)> {
    path::path_start_end_tangents(commands)
}

pub(super) fn path_midpoint_and_normal(
    commands: &[fret_core::PathCommand],
    bezier_steps: usize,
) -> Option<(Point, Point)> {
    path::path_midpoint_and_normal(commands, bezier_steps)
}

pub(super) fn step_wire_distance2(p: Point, from: Point, to: Point) -> f32 {
    step::step_wire_distance2(p, from, to)
}

pub(super) fn dist2_point_to_segment(p: Point, a: Point, b: Point) -> f32 {
    segment::dist2_point_to_segment(p, a, b)
}

fn closest_point_on_segment(p: Point, a: Point, b: Point) -> (Point, f32) {
    segment::closest_point_on_segment(p, a, b)
}
