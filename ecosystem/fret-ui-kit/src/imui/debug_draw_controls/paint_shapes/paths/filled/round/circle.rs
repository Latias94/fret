use fret_core::{Color, DrawOrder, Point, Px};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::paths::circle_path;

use super::super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_circle_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    center: Point,
    radius: Px,
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 || radius.0 <= 0.0 {
        return;
    }
    let commands = circle_path(center, radius);
    paint_path(
        painter,
        key,
        order,
        &commands,
        super::super::fill_style(),
        color,
        scale,
    );
}
