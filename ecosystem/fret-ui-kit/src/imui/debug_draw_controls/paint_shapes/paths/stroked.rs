mod beziers;
mod linear;
mod round;

pub(in crate::imui::debug_draw_controls::paint_shapes) use beziers::{
    paint_bezier_cubic, paint_bezier_quadratic,
};
pub(in crate::imui::debug_draw_controls::paint_shapes) use linear::{
    paint_line, paint_polyline, paint_quad, paint_rect, paint_triangle,
};
pub(in crate::imui::debug_draw_controls::paint_shapes) use round::{
    paint_circle, paint_ellipse, paint_ngon,
};
