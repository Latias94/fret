use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawCommand;
use crate::imui::debug_draw_controls::geometry::{
    points_are_finite, rect_is_empty, rect_is_finite, uv_points_are_finite,
};
use crate::imui::debug_draw_controls::paint_helpers::{
    corner_radii_are_visible, normalized_opacity, paint_image, paint_image_region,
    rounded_rect_corner_radii, uv_rect_is_valid,
};

pub(super) fn paint_debug_draw_media_command(
    painter: &mut CanvasPainter<'_>,
    key: u64,
    order: DrawOrder,
    command: &DebugDrawCommand,
) {
    match command {
        DebugDrawCommand::Image {
            rect,
            image,
            options,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0 || rect_is_empty(*rect) {
                return;
            }
            paint_image(painter, order, *rect, *image, *options, opacity);
        }
        DebugDrawCommand::ImageRegion {
            rect,
            image,
            uv,
            options,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0 || rect_is_empty(*rect) || !uv_rect_is_valid(*uv) {
                return;
            }
            paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
        }
        DebugDrawCommand::ImageQuad {
            image,
            points,
            uvs,
            options,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0
                || options.tint.a <= 0.0
                || !points_are_finite(points)
                || !uv_points_are_finite(uvs)
            {
                return;
            }
            painter.scene().push(fret_core::SceneOp::ImageQuad {
                order,
                points: *points,
                image: *image,
                uvs: *uvs,
                sampling: options.sampling,
                tint: options.tint,
                opacity,
            });
        }
        DebugDrawCommand::ImageRounded {
            rect,
            image,
            options,
            rounding,
            corners,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0 || rect_is_empty(*rect) || !rect_is_finite(*rect) {
                return;
            }
            let corner_radii = rounded_rect_corner_radii(*rect, *rounding, *corners);
            if corner_radii_are_visible(corner_radii) {
                painter.scene().push(fret_core::SceneOp::PushClipRRect {
                    rect: *rect,
                    corner_radii,
                });
                paint_image(painter, order, *rect, *image, *options, opacity);
                painter.scene().push(fret_core::SceneOp::PopClip);
            } else {
                paint_image(painter, order, *rect, *image, *options, opacity);
            }
        }
        DebugDrawCommand::ImageRegionRounded {
            rect,
            image,
            uv,
            options,
            rounding,
            corners,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0
                || rect_is_empty(*rect)
                || !rect_is_finite(*rect)
                || !uv_rect_is_valid(*uv)
            {
                return;
            }
            let corner_radii = rounded_rect_corner_radii(*rect, *rounding, *corners);
            if corner_radii_are_visible(corner_radii) {
                painter.scene().push(fret_core::SceneOp::PushClipRRect {
                    rect: *rect,
                    corner_radii,
                });
                paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
                painter.scene().push(fret_core::SceneOp::PopClip);
            } else {
                paint_image_region(painter, order, *rect, *image, *uv, *options, opacity);
            }
        }
        DebugDrawCommand::SvgImage { rect, svg, options } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0 || rect_is_empty(*rect) {
                return;
            }
            painter.svg_image(key, order, *rect, svg, options.fit, opacity);
        }
        DebugDrawCommand::SvgMaskIcon {
            rect,
            svg,
            color,
            options,
        } => {
            let opacity = normalized_opacity(options.opacity);
            if opacity <= 0.0 || color.a <= 0.0 || rect_is_empty(*rect) {
                return;
            }
            painter.svg_mask_icon(key, order, *rect, svg, options.fit, *color, opacity);
        }
        DebugDrawCommand::Line { .. }
        | DebugDrawCommand::Polyline { .. }
        | DebugDrawCommand::ConvexPolyFilled { .. }
        | DebugDrawCommand::ConcavePolyFilled { .. }
        | DebugDrawCommand::Rect { .. }
        | DebugDrawCommand::RectFilled { .. }
        | DebugDrawCommand::RectFilledMultiColor { .. }
        | DebugDrawCommand::Quad { .. }
        | DebugDrawCommand::QuadFilled { .. }
        | DebugDrawCommand::Triangle { .. }
        | DebugDrawCommand::TriangleFilled { .. }
        | DebugDrawCommand::TriangleMesh { .. }
        | DebugDrawCommand::ImageTriangleMesh { .. }
        | DebugDrawCommand::Circle { .. }
        | DebugDrawCommand::CircleFilled { .. }
        | DebugDrawCommand::Ngon { .. }
        | DebugDrawCommand::NgonFilled { .. }
        | DebugDrawCommand::Ellipse { .. }
        | DebugDrawCommand::EllipseFilled { .. }
        | DebugDrawCommand::BezierQuadratic { .. }
        | DebugDrawCommand::BezierCubic { .. }
        | DebugDrawCommand::PushClipRect { .. }
        | DebugDrawCommand::PopClipRect
        | DebugDrawCommand::Text { .. } => {}
    }
}
