use fret_core::Rect;

use super::super::ImUiDebugDrawList;
use super::super::commands::DebugDrawCommand;

impl ImUiDebugDrawList {
    pub fn push_clip_rect(&mut self, rect: Rect) {
        self.commands.push(DebugDrawCommand::PushClipRect { rect });
    }

    pub fn pop_clip_rect(&mut self) {
        self.commands.push(DebugDrawCommand::PopClipRect);
    }
}
