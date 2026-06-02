use fret_core::{Color, DrawOrder, Point, Px};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::paths::ngon_path;

use super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_ngon(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    center: Point,
    radius: Px,
    segments: usize,
    color: Color,
    style: DebugDrawStrokeStyle,
    scale: f32,
) {
    if color.a <= 0.0 || !style.is_visible() {
        return;
    }
    let Some(commands) = ngon_path(center, radius, segments) else {
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
