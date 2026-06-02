mod circle;
mod ellipse;
mod ngon;

pub(in crate::imui::debug_draw_controls::paint_shapes) use circle::paint_circle_filled;
pub(in crate::imui::debug_draw_controls::paint_shapes) use ellipse::paint_ellipse_filled;
pub(in crate::imui::debug_draw_controls::paint_shapes) use ngon::paint_ngon_filled;
