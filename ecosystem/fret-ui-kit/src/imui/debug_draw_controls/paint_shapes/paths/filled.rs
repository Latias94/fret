use fret_core::{Color, DrawOrder, FillStyle, PathStyle, Point, Px, Size};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::geometry::triangle_is_degenerate;
use crate::imui::debug_draw_controls::paths::{
    circle_path, concave_poly_fill_path, convex_poly_fill_path, ellipse_path, ngon_path, quad_path,
    triangle_path,
};

use super::common::paint_path;

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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
}

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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
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
    paint_path(painter, key, order, &commands, fill_style(), color, scale);
}

fn fill_style() -> PathStyle {
    PathStyle::Fill(FillStyle::default())
}
