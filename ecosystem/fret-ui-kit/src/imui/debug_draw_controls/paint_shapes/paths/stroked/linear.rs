use fret_core::{Color, DrawOrder, Point, Rect};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawStrokeStyle;
use crate::imui::debug_draw_controls::geometry::{rect_is_empty, triangle_is_degenerate};
use crate::imui::debug_draw_controls::paths::{polyline_path, quad_path, rect_path, triangle_path};

use super::super::common::paint_path;

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
