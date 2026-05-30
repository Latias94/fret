use super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::DebugDrawCommand;
use super::media;

pub(super) fn residual_summary(command: &DebugDrawCommand) -> Option<DebugDrawCommandSummary> {
    match command {
        DebugDrawCommand::ImageTriangleMesh {
            image,
            vertices,
            indices,
            ..
        } => Some(media::image_triangle_mesh_summary(
            *image,
            vertices.len(),
            indices.len(),
        )),
        DebugDrawCommand::PushClipRect { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::PushClipRect);
            summary.point_count = 4;
            Some(summary)
        }
        DebugDrawCommand::PopClipRect => Some(DebugDrawCommandSummary::new(
            DebugDrawCommandKind::PopClipRect,
        )),
        DebugDrawCommand::Image { image, .. } => Some(media::image_rect_summary(
            DebugDrawCommandKind::Image,
            *image,
        )),
        DebugDrawCommand::ImageRegion { image, .. } => Some(media::image_rect_summary(
            DebugDrawCommandKind::ImageRegion,
            *image,
        )),
        DebugDrawCommand::ImageQuad { image, .. } => Some(media::image_quad_summary(*image)),
        DebugDrawCommand::ImageRounded { image, .. } => Some(media::image_rect_summary(
            DebugDrawCommandKind::ImageRounded,
            *image,
        )),
        DebugDrawCommand::ImageRegionRounded { image, .. } => Some(media::image_rect_summary(
            DebugDrawCommandKind::ImageRegionRounded,
            *image,
        )),
        DebugDrawCommand::SvgImage { .. } => {
            Some(media::svg_rect_summary(DebugDrawCommandKind::SvgImage))
        }
        DebugDrawCommand::SvgMaskIcon { .. } => {
            Some(media::svg_rect_summary(DebugDrawCommandKind::SvgMaskIcon))
        }
        DebugDrawCommand::Text { .. } => {
            Some(DebugDrawCommandSummary::new(DebugDrawCommandKind::Text))
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
        | DebugDrawCommand::Circle { .. }
        | DebugDrawCommand::CircleFilled { .. }
        | DebugDrawCommand::Ngon { .. }
        | DebugDrawCommand::NgonFilled { .. }
        | DebugDrawCommand::Ellipse { .. }
        | DebugDrawCommand::EllipseFilled { .. }
        | DebugDrawCommand::BezierQuadratic { .. }
        | DebugDrawCommand::BezierCubic { .. } => None,
    }
}
