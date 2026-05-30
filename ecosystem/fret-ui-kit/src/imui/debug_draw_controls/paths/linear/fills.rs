use fret_core::{PathCommand, Point};

use super::polyline::polyline_path;

pub(in crate::imui::debug_draw_controls) fn convex_poly_fill_path(
    points: &[Point],
) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}

pub(in crate::imui::debug_draw_controls) fn concave_poly_fill_path(
    points: &[Point],
) -> Option<Vec<PathCommand>> {
    polyline_path(points, true)
}
