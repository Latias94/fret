mod line_poly;
mod rect_quad_triangle;

pub(in crate::imui::debug_draw_controls::paint_shapes) use line_poly::{
    paint_line, paint_polyline,
};
pub(in crate::imui::debug_draw_controls::paint_shapes) use rect_quad_triangle::{
    paint_quad, paint_rect, paint_triangle,
};
