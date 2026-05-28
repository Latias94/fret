use fret_core::{Color, DrawOrder, Point};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::paths::polyline_path;

use super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_line(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    from: Point,
    to: Point,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let commands = [
        fret_core::PathCommand::MoveTo(from),
        fret_core::PathCommand::LineTo(to),
    ];
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

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_polyline(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: &[Point],
    color: Color,
    style: DebugDrawStrokeStyle,
    closed: bool,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let Some(commands) = polyline_path(points, closed) else {
        return;
    };
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
