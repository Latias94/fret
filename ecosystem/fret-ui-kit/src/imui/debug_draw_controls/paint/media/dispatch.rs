use fret_core::DrawOrder;
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawCommand;

use super::{MediaPaintKey, raster, rounded, svg};

pub(in crate::imui::debug_draw_controls::paint) fn paint_debug_draw_media_command(
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
        } => raster::paint_image_command(painter, order, *rect, *image, *options),
        DebugDrawCommand::ImageRegion {
            rect,
            image,
            uv,
            options,
        } => raster::paint_image_region_command(painter, order, *rect, *image, *uv, *options),
        DebugDrawCommand::ImageQuad {
            image,
            points,
            uvs,
            options,
        } => raster::paint_image_quad_command(painter, order, *image, *points, *uvs, *options),
        DebugDrawCommand::ImageRounded {
            rect,
            image,
            options,
            rounding,
            corners,
        } => rounded::paint_image_rounded_command(
            painter, order, *rect, *image, *options, *rounding, *corners,
        ),
        DebugDrawCommand::ImageRegionRounded {
            rect,
            image,
            uv,
            options,
            rounding,
            corners,
        } => rounded::paint_image_region_rounded_command(
            painter, order, *rect, *image, *uv, *options, *rounding, *corners,
        ),
        DebugDrawCommand::SvgImage { rect, svg, options } => svg::paint_svg_image_command(
            painter,
            MediaPaintKey {
                key,
                order,
                rect: *rect,
            },
            svg,
            *options,
        ),
        DebugDrawCommand::SvgMaskIcon {
            rect,
            svg,
            color,
            options,
        } => svg::paint_svg_mask_icon_command(
            painter,
            MediaPaintKey {
                key,
                order,
                rect: *rect,
            },
            svg,
            *color,
            *options,
        ),
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
