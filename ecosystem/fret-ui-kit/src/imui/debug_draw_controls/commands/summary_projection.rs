use fret_core::Rect;

use super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::DebugDrawCommand;

mod clip_state;
mod geometry;
mod media;

impl DebugDrawCommand {
    pub(in crate::imui::debug_draw_controls) fn summary_with_clip_state(
        &self,
        channel: Option<usize>,
        clip_stack: &mut Vec<Rect>,
    ) -> DebugDrawCommandSummary {
        let mut summary = self.summary().with_channel(channel);
        clip_state::apply_clip_state(self, clip_stack, &mut summary);
        summary
    }

    fn summary(&self) -> DebugDrawCommandSummary {
        if let Some(summary) = geometry::geometry_summary(self) {
            return summary;
        }

        match self {
            DebugDrawCommand::ImageTriangleMesh {
                image,
                vertices,
                indices,
                ..
            } => media::image_triangle_mesh_summary(*image, vertices.len(), indices.len()),
            DebugDrawCommand::PushClipRect { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::PushClipRect);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::PopClipRect => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::PopClipRect)
            }
            DebugDrawCommand::Image { image, .. } => {
                media::image_rect_summary(DebugDrawCommandKind::Image, *image)
            }
            DebugDrawCommand::ImageRegion { image, .. } => {
                media::image_rect_summary(DebugDrawCommandKind::ImageRegion, *image)
            }
            DebugDrawCommand::ImageQuad { image, .. } => media::image_quad_summary(*image),
            DebugDrawCommand::ImageRounded { image, .. } => {
                media::image_rect_summary(DebugDrawCommandKind::ImageRounded, *image)
            }
            DebugDrawCommand::ImageRegionRounded { image, .. } => {
                media::image_rect_summary(DebugDrawCommandKind::ImageRegionRounded, *image)
            }
            DebugDrawCommand::SvgImage { .. } => {
                media::svg_rect_summary(DebugDrawCommandKind::SvgImage)
            }
            DebugDrawCommand::SvgMaskIcon { .. } => {
                media::svg_rect_summary(DebugDrawCommandKind::SvgMaskIcon)
            }
            DebugDrawCommand::Text { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::Text)
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
            | DebugDrawCommand::BezierCubic { .. } => {
                unreachable!("geometry commands are handled by geometry_summary")
            }
        }
    }
}
