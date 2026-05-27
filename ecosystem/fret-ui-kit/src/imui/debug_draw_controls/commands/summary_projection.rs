use fret_core::Rect;

use super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::DebugDrawCommand;

mod media;

impl DebugDrawCommand {
    pub(in crate::imui::debug_draw_controls) fn summary_with_clip_state(
        &self,
        channel: Option<usize>,
        clip_stack: &mut Vec<Rect>,
    ) -> DebugDrawCommandSummary {
        let mut summary = self.summary().with_channel(channel);
        match self {
            DebugDrawCommand::PushClipRect { rect } => {
                clip_stack.push(*rect);
                summary.clip_rect = Some(*rect);
            }
            DebugDrawCommand::PopClipRect => {
                clip_stack.pop();
                summary.clip_rect = clip_stack.last().copied();
            }
            _ => {
                summary.clip_rect = clip_stack.last().copied();
            }
        }
        summary.clip_depth = clip_stack.len();
        summary
    }

    fn summary(&self) -> DebugDrawCommandSummary {
        match self {
            DebugDrawCommand::Line { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Line);
                summary.point_count = 2;
                summary
            }
            DebugDrawCommand::Polyline { points, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Polyline);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::ConvexPolyFilled { points, .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ConvexPolyFilled);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::ConcavePolyFilled { points, .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::ConcavePolyFilled);
                summary.point_count = points.len();
                summary
            }
            DebugDrawCommand::Rect { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Rect);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::RectFilled { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilled);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::RectFilledMultiColor { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilledMultiColor);
                summary.point_count = 4;
                summary.vertex_count = 4;
                summary.index_count = 6;
                summary.triangle_count = 2;
                summary
            }
            DebugDrawCommand::Quad { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Quad);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::QuadFilled { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::QuadFilled);
                summary.point_count = 4;
                summary
            }
            DebugDrawCommand::Triangle { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Triangle);
                summary.point_count = 3;
                summary
            }
            DebugDrawCommand::TriangleFilled { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleFilled);
                summary.point_count = 3;
                summary.triangle_count = 1;
                summary
            }
            DebugDrawCommand::TriangleMesh { vertices, indices } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleMesh);
                summary.vertex_count = vertices.len();
                summary.index_count = indices.len();
                summary.triangle_count = indices.len() / 3;
                summary
            }
            DebugDrawCommand::ImageTriangleMesh {
                image,
                vertices,
                indices,
                ..
            } => media::image_triangle_mesh_summary(*image, vertices.len(), indices.len()),
            DebugDrawCommand::Circle { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::Circle)
            }
            DebugDrawCommand::CircleFilled { .. } => {
                DebugDrawCommandSummary::new(DebugDrawCommandKind::CircleFilled)
            }
            DebugDrawCommand::Ngon { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ngon);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::NgonFilled { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::NgonFilled);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::Ellipse { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ellipse);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::EllipseFilled { segments, .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::EllipseFilled);
                summary.point_count = *segments;
                summary
            }
            DebugDrawCommand::BezierQuadratic { .. } => {
                let mut summary =
                    DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierQuadratic);
                summary.point_count = 3;
                summary
            }
            DebugDrawCommand::BezierCubic { .. } => {
                let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierCubic);
                summary.point_count = 4;
                summary
            }
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
        }
    }
}
