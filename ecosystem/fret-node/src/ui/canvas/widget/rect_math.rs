use fret_core::PathCommand;

use super::*;

pub(super) fn rect_from_points(a: Point, b: Point) -> Rect {
    super::rect_math_core::rect_from_points(a, b)
}

pub(super) fn rect_union(a: Rect, b: Rect) -> Rect {
    super::rect_math_core::rect_union(a, b)
}

pub(super) fn rects_intersect(a: Rect, b: Rect) -> bool {
    super::rect_math_core::rects_intersect(a, b)
}

pub(super) fn inflate_rect(rect: Rect, margin: f32) -> Rect {
    super::rect_math_core::inflate_rect(rect, margin)
}

pub(super) fn path_bounds_rect(commands: &[PathCommand]) -> Option<Rect> {
    super::rect_math_path::path_bounds_rect(commands)
}

pub(super) fn edge_bounds_rect(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    pad: f32,
) -> Rect {
    super::rect_math_path::edge_bounds_rect(route, from, to, zoom, pad)
}
