use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::DebugDrawCommand;

pub(super) fn bezier_geometry_summary(
    command: &DebugDrawCommand,
) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
        DebugDrawCommand::BezierQuadratic { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierQuadratic);
            summary.point_count = 3;
            summary
        }
        DebugDrawCommand::BezierCubic { .. } => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::BezierCubic);
            summary.point_count = 4;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
