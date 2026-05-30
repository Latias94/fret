use fret_core::{Color, DrawOrder, Point};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::geometry::triangle_is_degenerate;
use crate::imui::debug_draw_controls::paths::{
    concave_poly_fill_path, convex_poly_fill_path, quad_path, triangle_path,
};

use super::super::common::paint_path;

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_convex_poly_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: &[Point],
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 {
        return;
    }
    let Some(commands) = convex_poly_fill_path(points) else {
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

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_concave_poly_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: &[Point],
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 {
        return;
    }
    let Some(commands) = concave_poly_fill_path(points) else {
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

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_quad_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: [Point; 4],
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 {
        return;
    }
    let commands = quad_path(points[0], points[1], points[2], points[3]);
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

pub(in crate::imui::debug_draw_controls::paint_shapes) fn paint_triangle_filled(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    points: [Point; 3],
    color: Color,
    scale: f32,
) {
    if color.a <= 0.0 || triangle_is_degenerate(points[0], points[1], points[2]) {
        return;
    }
    let commands = triangle_path(points[0], points[1], points[2]);
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
