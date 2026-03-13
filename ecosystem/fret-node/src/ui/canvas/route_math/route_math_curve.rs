use super::*;

pub(crate) fn wire_ctrl_points(from: Point, to: Point, zoom: f32) -> (Point, Point) {
    canvas_wires::wire_ctrl_points(from, to, zoom)
}

pub(crate) fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    canvas_wires::cubic_bezier(p0, p1, p2, p3, t)
}

pub(crate) fn cubic_bezier_derivative(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    canvas_wires::cubic_bezier_derivative(p0, p1, p2, p3, t)
}

pub(crate) fn normal_from_tangent(tangent: Point) -> Point {
    canvas_wires::normal_from_tangent(tangent)
}
