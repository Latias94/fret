use fret_core::{FillStyle, PathStyle};

mod polygons;
mod round;

pub(in crate::imui::debug_draw_controls::paint_shapes) use polygons::{
    paint_concave_poly_filled, paint_convex_poly_filled, paint_quad_filled, paint_triangle_filled,
};
pub(in crate::imui::debug_draw_controls::paint_shapes) use round::{
    paint_circle_filled, paint_ellipse_filled, paint_ngon_filled,
};

fn fill_style() -> PathStyle {
    PathStyle::Fill(FillStyle::default())
}
