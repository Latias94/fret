use fret_core::{DrawOrder, ImageId, Rect, UvRect};
use fret_ui::canvas::CanvasPainter;

use crate::imui::debug_draw_controls::DebugDrawCommand;

mod raster;
mod rounded;
mod svg;

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

#[derive(Debug, Clone, Copy)]
pub(super) struct MediaPaintKey {
    pub(super) key: u64,
    pub(super) order: DrawOrder,
    pub(super) rect: Rect,
}

pub(super) type RasterImage = ImageId;
pub(super) type RasterUvRect = UvRect;
