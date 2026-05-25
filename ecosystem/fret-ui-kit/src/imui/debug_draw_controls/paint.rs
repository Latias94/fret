use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use super::DebugDrawCommand;
use super::geometry::{points_are_finite, rect_is_empty, rect_is_finite, uv_points_are_finite};
use super::paint_helpers::{
    corner_radii_are_visible, normalized_opacity, paint_image, paint_image_region,
    rounded_rect_corner_radii, uv_rect_is_valid,
};
use super::paint_shapes::paint_debug_draw_shape_command;

pub(super) fn paint_debug_draw_commands(
    painter: &mut CanvasPainter<'_>,
    commands: &[DebugDrawCommand],
) {
    let scale = painter.scale_factor().max(1.0);
    let mut open_clip_depth = 0usize;
    for (index, command) in commands.iter().enumerate() {
        let order = DrawOrder(index as u32);
        let key = painter.key(&("fret-ui-kit.imui.debug_draw.command", index));
        match command {
            DebugDrawCommand::PushClipRect { rect } => {
                if rect_is_empty(*rect) {
                    continue;
                }
                painter
                    .scene()
                    .push(fret_core::SceneOp::PushClipRect { rect: *rect });
                open_clip_depth += 1;
            }
            DebugDrawCommand::PopClipRect => {
                if open_clip_depth == 0 {
                    continue;
                }
                painter.scene().push(fret_core::SceneOp::PopClip);
                open_clip_depth -= 1;
            }
            DebugDrawCommand::Image {
                rect,
                image,
                options,
            } => {
                let opacity = normalized_opacity(options.opacity);
                if opacity <= 0.0 || rect_is_empty(*rect) {
                    continue;
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
                    continue;
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
                    continue;
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
                    continue;
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
                    continue;
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
                    continue;
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
                    continue;
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
            | DebugDrawCommand::Text { .. } => {
                paint_debug_draw_shape_command(painter, index, command, scale);
            }
        }
    }

    for _ in 0..open_clip_depth {
        painter.scene().push(fret_core::SceneOp::PopClip);
    }
}
