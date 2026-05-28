mod arcs;
mod beziers;
mod segments;

pub(in crate::imui::debug_draw_controls) use arcs::{
    append_arc_points, append_elliptical_arc_points,
};
pub(in crate::imui::debug_draw_controls) use beziers::{
    cubic_bezier_point, quadratic_bezier_point,
};
pub(in crate::imui::debug_draw_controls) use segments::{
    path_arc_segments, path_bezier_segments, path_elliptical_arc_segments,
};
