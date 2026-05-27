use fret_core::{DrawOrder, Point, UvPoint};
use fret_ui::canvas::CanvasPainter;

use super::{RasterImage, RasterUvRect};
use crate::imui::debug_draw_controls::geometry::{
    points_are_finite, rect_is_empty, uv_points_are_finite,
};
use crate::imui::debug_draw_controls::paint_helpers::{
    normalized_opacity, paint_image, paint_image_region, uv_rect_is_valid,
};
use crate::imui::debug_draw_controls::{DebugDrawImageOptions, DebugDrawImageQuadOptions};

pub(super) fn paint_image_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: fret_core::Rect,
    image: RasterImage,
    options: DebugDrawImageOptions,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || rect_is_empty(rect) {
        return;
    }
    paint_image(painter, order, rect, image, options, opacity);
}

pub(super) fn paint_image_region_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: fret_core::Rect,
    image: RasterImage,
    uv: RasterUvRect,
    options: DebugDrawImageOptions,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || rect_is_empty(rect) || !uv_rect_is_valid(uv) {
        return;
    }
    paint_image_region(painter, order, rect, image, uv, options, opacity);
}

pub(super) fn paint_image_quad_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    image: RasterImage,
    points: [Point; 4],
    uvs: [UvPoint; 4],
    options: DebugDrawImageQuadOptions,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0
        || options.tint.a <= 0.0
        || !points_are_finite(&points)
        || !uv_points_are_finite(&uvs)
    {
        return;
    }
    painter.scene().push(fret_core::SceneOp::ImageQuad {
        order,
        points,
        image,
        uvs,
        sampling: options.sampling,
        tint: options.tint,
        opacity,
    });
}
