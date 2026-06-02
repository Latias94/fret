use fret_core::{Color, DrawOrder, Point, Size};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::paths::ellipse_path;

use super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_ellipse(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    center: Point,
    radius: Size,
    rotation_radians: f32,
    segments: usize,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let Some(commands) = ellipse_path(center, radius, rotation_radians, segments) else {
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
