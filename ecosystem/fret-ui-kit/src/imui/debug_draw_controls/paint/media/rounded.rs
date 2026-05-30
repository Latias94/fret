use fret_core::{DrawOrder, Px, Rect};
use fret_ui::canvas::CanvasPainter;

use super::{RasterImage, RasterUvRect};
use crate::imui::debug_draw_controls::geometry::{rect_is_empty, rect_is_finite};
use crate::imui::debug_draw_controls::paint_helpers::{
    corner_radii_are_visible, normalized_opacity, paint_image, paint_image_region,
    rounded_rect_corner_radii, uv_rect_is_valid,
};
use crate::imui::debug_draw_controls::{DebugDrawImageOptions, DebugDrawRoundCorners};

pub(super) fn paint_image_rounded_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: RasterImage,
    options: DebugDrawImageOptions,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || rect_is_empty(rect) || !rect_is_finite(rect) {
        return;
    }
    let corner_radii = rounded_rect_corner_radii(rect, rounding, corners);
    if corner_radii_are_visible(corner_radii) {
        painter
            .scene()
            .push(fret_core::SceneOp::PushClipRRect { rect, corner_radii });
        paint_image(painter, order, rect, image, options, opacity);
        painter.scene().push(fret_core::SceneOp::PopClip);
    } else {
        paint_image(painter, order, rect, image, options, opacity);
    }
}

pub(super) fn paint_image_region_rounded_command(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: RasterImage,
    uv: RasterUvRect,
    options: DebugDrawImageOptions,
    rounding: Px,
    corners: DebugDrawRoundCorners,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || rect_is_empty(rect) || !rect_is_finite(rect) || !uv_rect_is_valid(uv) {
        return;
    }
    let corner_radii = rounded_rect_corner_radii(rect, rounding, corners);
    if corner_radii_are_visible(corner_radii) {
        painter
            .scene()
            .push(fret_core::SceneOp::PushClipRRect { rect, corner_radii });
        paint_image_region(painter, order, rect, image, uv, options, opacity);
        painter.scene().push(fret_core::SceneOp::PopClip);
    } else {
        paint_image_region(painter, order, rect, image, uv, options, opacity);
    }
}
