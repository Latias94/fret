use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::{DebugDrawCommand, DebugDrawLinearCommand};

pub(super) fn linear_geometry_summary(
    command: &DebugDrawCommand,
) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Line { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Line);
            summary.point_count = 2;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Polyline { points, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Polyline);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::ConvexPolyFilled { points, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConvexPolyFilled);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::ConcavePolyFilled { points, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::ConcavePolyFilled);
            summary.point_count = points.len();
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Rect { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Rect);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::RectFilled { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilled);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::RectFilledMultiColor { .. }) => {
            let mut summary =
                DebugDrawCommandSummary::new(DebugDrawCommandKind::RectFilledMultiColor);
            summary.point_count = 4;
            summary.vertex_count = 4;
            summary.index_count = 6;
            summary.triangle_count = 2;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Quad { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Quad);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::QuadFilled { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::QuadFilled);
            summary.point_count = 4;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::Triangle { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Triangle);
            summary.point_count = 3;
            summary
        }
        DebugDrawCommand::Linear(DebugDrawLinearCommand::TriangleFilled { .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::TriangleFilled);
            summary.point_count = 3;
            summary.triangle_count = 1;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
