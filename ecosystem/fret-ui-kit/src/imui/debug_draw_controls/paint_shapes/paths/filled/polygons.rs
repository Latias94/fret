mod multi;
mod primitives;

pub(in crate::imui::debug_draw_controls::paint_shapes) use multi::{
    paint_concave_poly_filled, paint_convex_poly_filled,
};
pub(in crate::imui::debug_draw_controls::paint_shapes) use primitives::{
    paint_quad_filled, paint_triangle_filled,
};
