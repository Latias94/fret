use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::DebugDrawCommand;

pub(super) fn round_geometry_summary(
    command: &DebugDrawCommand,
) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
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
        _ => return None,
    };
    Some(summary)
}
