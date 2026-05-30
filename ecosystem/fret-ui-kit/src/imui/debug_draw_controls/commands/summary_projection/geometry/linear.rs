use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::DebugDrawCommand;

pub(super) fn linear_geometry_summary(
    command: &DebugDrawCommand,
) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
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
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConvexPolyFilled);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::ConcavePolyFilled { points, .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConcavePolyFilled);
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
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleFilled);
            summary.point_count = 3;
            summary.triangle_count = 1;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
