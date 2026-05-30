use fret_core::Color;
use fret_ui::SvgSource;
use fret_ui::canvas::CanvasPainter;

use super::MediaPaintKey;
use crate::imui::debug_draw_controls::DebugDrawSvgOptions;
use crate::imui::debug_draw_controls::geometry::rect_is_empty;
use crate::imui::debug_draw_controls::paint_helpers::normalized_opacity;

pub(super) fn paint_svg_image_command(
    painter: &mut CanvasPainter<'_>,
    paint: MediaPaintKey,
    svg: &SvgSource,
    options: DebugDrawSvgOptions,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || rect_is_empty(paint.rect) {
        return;
    }
    painter.svg_image(
        paint.key,
        paint.order,
        paint.rect,
        svg,
        options.fit,
        opacity,
    );
}

pub(super) fn paint_svg_mask_icon_command(
    painter: &mut CanvasPainter<'_>,
    paint: MediaPaintKey,
    svg: &SvgSource,
    color: Color,
    options: DebugDrawSvgOptions,
) {
    let opacity = normalized_opacity(options.opacity);
    if opacity <= 0.0 || color.a <= 0.0 || rect_is_empty(paint.rect) {
        return;
    }
    painter.svg_mask_icon(
        paint.key,
        paint.order,
        paint.rect,
        svg,
        options.fit,
        color,
        opacity,
    );
}
