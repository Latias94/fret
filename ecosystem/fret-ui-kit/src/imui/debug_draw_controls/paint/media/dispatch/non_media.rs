use crate::imui::debug_draw_controls::DebugDrawCommand;

pub(super) fn is_non_media_command(command: &DebugDrawCommand) -> bool {
    match command {
        DebugDrawCommand::Media(_) => false,
        DebugDrawCommand::Linear(_)
        | DebugDrawCommand::Mesh(_)
        | DebugDrawCommand::Round(_)
        | DebugDrawCommand::BezierQuadratic { .. }
        | DebugDrawCommand::BezierCubic { .. }
        | DebugDrawCommand::Clip(_)
        | DebugDrawCommand::Text { .. } => true,
    }
}
