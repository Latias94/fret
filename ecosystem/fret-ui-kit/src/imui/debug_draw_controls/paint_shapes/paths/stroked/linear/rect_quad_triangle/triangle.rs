use fret_core::{Color, DrawOrder, Point};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::geometry::triangle_is_degenerate;
use crate::imui::debug_draw_controls::paths::triangle_path;

use super::super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_triangle(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: [Point; 3],
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0
        || !style.is_visible()
        || triangle_is_degenerate(points[0], points[1], points[2])
    {
        return;
    }
    let commands = triangle_path(points[0], points[1], points[2]);
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
