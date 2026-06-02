use fret_core::{Color, DrawOrder, Point};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::paths::quad_path;

use super::super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_quad(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: [Point; 4],
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let commands = quad_path(points[0], points[1], points[2], points[3]);
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
