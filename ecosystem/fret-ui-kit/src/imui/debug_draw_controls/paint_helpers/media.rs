use fret_core::{DrawOrder, ImageId, Rect, UvRect};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawImageOptions;

pub(in crate::imui::debug_draw_controls) fn normalized_opacity(opacity: f32) -> f32 {
    if opacity.is_finite() {
        opacity.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

pub(in crate::imui::debug_draw_controls) fn uv_rect_is_valid(uv: UvRect) -> bool {
    uv.u0.is_finite()
        && uv.v0.is_finite()
        && uv.u1.is_finite()
        && uv.v1.is_finite()
        && uv.u1 > uv.u0
        && uv.v1 > uv.v0
}

pub(in crate::imui::debug_draw_controls) fn paint_image(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: ImageId,
    options: DebugDrawImageOptions,
    opacity: f32,
) {
    painter.scene().push(fret_core::SceneOp::Image {
        order,
        rect,
        image,
        fit: options.fit,
        sampling: options.sampling,
        opacity,
    });
}

pub(in crate::imui::debug_draw_controls) fn paint_image_region(
    painter: &mut CanvasPainter<'_>,
    order: DrawOrder,
    rect: Rect,
    image: ImageId,
    uv: UvRect,
    options: DebugDrawImageOptions,
    opacity: f32,
) {
    painter.scene().push(fret_core::SceneOp::ImageRegion {
        order,
        rect,
        image,
        uv,
        sampling: options.sampling,
        opacity,
    });
}
