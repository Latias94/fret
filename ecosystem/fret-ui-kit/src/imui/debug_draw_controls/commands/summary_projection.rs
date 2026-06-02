use fret_core::Rect;

use super::super::summaries::DebugDrawCommandSummary;
use super::{DebugDrawCommand, DebugDrawMeshCommand};

mod clip_state;
mod geometry;
mod media;
mod residual;

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
        if let Some(summary) = residual::residual_summary(self) {
            return summary;
        }

        match self {
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
            | DebugDrawCommand::Mesh(DebugDrawMeshCommand::TriangleMesh { .. })
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
            DebugDrawCommand::Mesh(DebugDrawMeshCommand::ImageTriangleMesh { .. })
            | DebugDrawCommand::Clip(_)
            | DebugDrawCommand::Media(_)
            | DebugDrawCommand::Text { .. } => {
                unreachable!("residual commands are handled by residual_summary")
            }
        }
    }
}
