use fret_core::{Color, DrawOrder, Rect};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::geometry::rect_is_empty;
use crate::imui::debug_draw_controls::paths::rect_path;

use super::super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_rect(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    rect: Rect,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() || rect_is_empty(rect) {
        return;
    }
    let commands = rect_path(rect);
    paint_path(
        painter,
        key,
        order,
        &commands,
        style.path_style(),
        color,
        scale,
    );
}
