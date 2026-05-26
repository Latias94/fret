use super::super::ImUiDebugDrawList;
use super::super::summaries::{DebugDrawCommandSummary, DebugDrawListSummary};

impl ImUiDebugDrawList {
    /// Return command summaries in the order the list would paint after channel merge.
    pub fn command_summaries(&self) -> Vec<DebugDrawCommandSummary> {
        let mut summaries = Vec::with_capacity(self.command_count());
        let mut clip_stack = Vec::new();
        self.for_each_command_with_channel(|channel, command| {
            summaries.push(command.summary_with_clip_state(channel, &mut clip_stack));
        });
        summaries
    }

    /// Return aggregate source-level metadata for recorded debug draw commands.
    pub fn list_summary(&self) -> DebugDrawListSummary {
        let mut summary = DebugDrawListSummary::new();
        let mut clip_stack = Vec::new();
        self.for_each_command_with_channel(|channel, command| {
            summary.include(command.summary_with_clip_state(channel, &mut clip_stack));
        });
        summary.set_final_clip_depth(clip_stack.len());
        summary
    }
}
