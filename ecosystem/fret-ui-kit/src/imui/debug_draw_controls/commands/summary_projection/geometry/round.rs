use super::super::super::super::summaries::{DebugDrawCommandKind, DebugDrawCommandSummary};
use super::super::super::{DebugDrawCommand, DebugDrawRoundCommand};

pub(super) fn round_geometry_summary(
    command: &DebugDrawCommand,
) -> Option<DebugDrawCommandSummary> {
    let summary = match command {
        DebugDrawCommand::Round(DebugDrawRoundCommand::Circle { .. }) => {
            DebugDrawCommandSummary::new(DebugDrawCommandKind::Circle)
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::CircleFilled { .. }) => {
            DebugDrawCommandSummary::new(DebugDrawCommandKind::CircleFilled)
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::Ngon { segments, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ngon);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::NgonFilled { segments, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::NgonFilled);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::Ellipse { segments, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::Ellipse);
            summary.point_count = *segments;
            summary
        }
        DebugDrawCommand::Round(DebugDrawRoundCommand::EllipseFilled { segments, .. }) => {
            let mut summary = DebugDrawCommandSummary::new(DebugDrawCommandKind::EllipseFilled);
            summary.point_count = *segments;
            summary
        }
        _ => return None,
    };
    Some(summary)
}
