use super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::{
    DebugDrawClipCommand, DebugDrawCommand, DebugDrawMediaCommand, DebugDrawMeshCommand,
};
use super::media;

pub(super) fn residual_summary(command: &DebugDrawCommand) -> Option<DebugDrawCommandSummary> {
    match command {
        DebugDrawCommand::Mesh(DebugDrawMeshCommand::ImageTriangleMesh {
            image,
            vertices,
            indices,
            ..
        }) => Some(media::image_triangle_mesh_summary(
            *image,
            vertices.len(),
            indices.len(),
        )),
        DebugDrawCommand::Clip(DebugDrawClipCommand::PushClipRect { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::PushClipRect);
            summary.point_count = 4;
            Some(summary)
        }
        DebugDrawCommand::Clip(DebugDrawClipCommand::PopClipRect) => Some(
            DebugDrawCommandSummary::new(DebugDrawCommandKind::PopClipRect),
        ),
        DebugDrawCommand::Media(DebugDrawMediaCommand::Image { image, .. }) => Some(
            media::image_rect_summary(DebugDrawCommandKind::Image, *image),
        ),
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegion { image, .. }) => Some(
            media::image_rect_summary(DebugDrawCommandKind::ImageRegion, *image),
        ),
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageQuad { image, .. }) => {
            Some(media::image_quad_summary(*image))
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRounded { image, .. }) => Some(
            media::image_rect_summary(DebugDrawCommandKind::ImageRounded, *image),
        ),
        DebugDrawCommand::Media(DebugDrawMediaCommand::ImageRegionRounded { image, .. }) => Some(
            media::image_rect_summary(DebugDrawCommandKind::ImageRegionRounded, *image),
        ),
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgImage { .. }) => {
            Some(media::svg_rect_summary(DebugDrawCommandKind::SvgImage))
        }
        DebugDrawCommand::Media(DebugDrawMediaCommand::SvgMaskIcon { .. }) => {
            Some(media::svg_rect_summary(DebugDrawCommandKind::SvgMaskIcon))
        }
        DebugDrawCommand::Text { .. } => {
            Some(DebugDrawCommandSummary::new(DebugDrawCommandKind::Text))
        }
        DebugDrawCommand::Linear(_)
        | DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh { .. })
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
