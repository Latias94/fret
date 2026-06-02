use fret_core::Rect;

use super::super::super::summaries::DebugDrawCommandSummary;
use super::super::{DebugDrawClipCommand, DebugDrawCommand};

pub(super) fn apply_clip_state(
    command: &DebugDrawCommand,
    clip_stack: &mut Vec<Rect>,
    summary: &mut DebugDrawCommandSummary,
) {
    match command {
        DebugDrawCommand::Clip(DebugDrawClipCommand::PushClipRect { rect }) => {
            clip_stack.push(*rect);
            summary.clip_rect = Some(*rect);
        }
        DebugDrawCommand::Clip(DebugDrawClipCommand::PopClipRect) => {
            clip_stack.pop();
            summary.clip_rect = clip_stack.last().copied();
        }
        _ => {
            summary.clip_rect = clip_stack.last().copied();
        }
    }
    summary.clip_depth = clip_stack.len();
}
