use fret_core::{Point, Rect};

pub(super) fn rect_from_points(a: Point, b: Point) -> Rect {
    fret_canvas::view::rect_from_points(a, b)
}

pub(super) fn rect_union(a: Rect, b: Rect) -> Rect {
    fret_canvas::view::rect_union(a, b)
}

pub(super) fn rects_intersect(a: Rect, b: Rect) -> bool {
    fret_canvas::view::rects_intersect(a, b)
}

pub(super) fn inflate_rect(rect: Rect, margin: f32) -> Rect {
    fret_canvas::view::inflate_rect(rect, margin)
}

#[cfg(test)]
mod tests;
