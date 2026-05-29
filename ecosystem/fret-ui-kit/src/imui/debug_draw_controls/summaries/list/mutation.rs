use super::super::DebugDrawCommandSummary;
use super::DebugDrawListSummary;
use super::classification::{DebugDrawListSummaryClass, classify_debug_draw_summary_kind};

impl DebugDrawListSummary {
    pub(in crate::imui::debug_draw_controls) fn new() -> Self {
        Self {
            command_count: 0,
            clip_push_count: 0,
            clip_pop_count: 0,
            max_clip_depth: 0,
            final_clip_depth: 0,
            image_command_count: 0,
            svg_command_count: 0,
            text_command_count: 0,
            point_count: 0,
            vertex_count: 0,
            index_count: 0,
            triangle_count: 0,
        }
    }

    pub(in crate::imui::debug_draw_controls) fn set_final_clip_depth(
        &mut self,
        final_clip_depth: usize,
    ) {
        self.final_clip_depth = final_clip_depth;
    }

    pub(in crate::imui::debug_draw_controls) fn include(
        &mut self,
        command: DebugDrawCommandSummary,
    ) {
        self.command_count += 1;
        self.point_count += command.point_count;
        self.vertex_count += command.vertex_count;
        self.index_count += command.index_count;
        self.triangle_count += command.triangle_count;
        self.max_clip_depth = self.max_clip_depth.max(command.clip_depth);

        match classify_debug_draw_summary_kind(command.kind) {
            DebugDrawListSummaryClass::ClipPush => self.clip_push_count += 1,
            DebugDrawListSummaryClass::ClipPop => self.clip_pop_count += 1,
            DebugDrawListSummaryClass::Image => self.image_command_count += 1,
            DebugDrawListSummaryClass::Svg => self.svg_command_count += 1,
            DebugDrawListSummaryClass::Text => self.text_command_count += 1,
            DebugDrawListSummaryClass::Geometry => {}
        }
    }
}
