use fret_core::{Color, DrawOrder, Point};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::paths::{bezier_cubic_path, bezier_quadratic_path};

use super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_bezier_quadratic(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    from: Point,
    ctrl: Point,
    to: Point,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let commands = bezier_quadratic_path(from, ctrl, to);
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

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_bezier_cubic(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    from: Point,
    ctrl1: Point,
    ctrl2: Point,
    to: Point,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let commands = bezier_cubic_path(from, ctrl1, ctrl2, to);
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
