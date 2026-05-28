use fret_core::{Color, DrawOrder, Point, Px, Size};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::paths::{circle_path, ellipse_path, ngon_path};

use super::super::common::paint_path;

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
        super::fill_style(),
        color,
        scale,
    );
}

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_ngon_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    center: Point,
    radius: Px,
    segments: usize,
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 {
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
        super::fill_style(),
        color,
        scale,
    );
}

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_ellipse_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    center: Point,
    radius: Size,
    rotation_radians: f32,
    segments: usize,
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 {
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
        super::fill_style(),
        color,
        scale,
    );
}
