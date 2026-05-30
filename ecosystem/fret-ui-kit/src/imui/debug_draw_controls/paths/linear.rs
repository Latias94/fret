mod fills;
mod polyline;
mod primitives;

pub(in crate::imui::debug_draw_controls) use fills::{
    concave_poly_fill_path, convex_poly_fill_path,
};
pub(in crate::imui::debug_draw_controls) use polyline::{
    path_stroke_required_points, polyline_path,
};
pub(in crate::imui::debug_draw_controls) use primitives::{quad_path, triangle_path};
