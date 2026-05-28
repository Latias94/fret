use super::DebugDrawCommandSummary;

mod classification;

use classification::{DebugDrawListSummaryClass, classify_debug_draw_summary_kind};

/// Aggregate source-level metadata for an IMUI debug draw list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DebugDrawListSummary {
    command_count: usize,
    clip_push_count: usize,
    clip_pop_count: usize,
    max_clip_depth: usize,
    final_clip_depth: usize,
    image_command_count: usize,
    svg_command_count: usize,
    text_command_count: usize,
    point_count: usize,
    vertex_count: usize,
    index_count: usize,
    triangle_count: usize,
}

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

    pub fn command_count(self) -> usize {
        self.command_count
    }

    pub fn clip_push_count(self) -> usize {
        self.clip_push_count
    }

    pub fn clip_pop_count(self) -> usize {
        self.clip_pop_count
    }

    pub fn max_clip_depth(self) -> usize {
        self.max_clip_depth
    }

    pub fn final_clip_depth(self) -> usize {
        self.final_clip_depth
    }

    pub fn image_command_count(self) -> usize {
        self.image_command_count
    }

    pub fn svg_command_count(self) -> usize {
        self.svg_command_count
    }

    pub fn text_command_count(self) -> usize {
        self.text_command_count
    }

    pub fn point_count(self) -> usize {
        self.point_count
    }

    pub fn vertex_count(self) -> usize {
        self.vertex_count
    }

    pub fn index_count(self) -> usize {
        self.index_count
    }

    pub fn triangle_count(self) -> usize {
        self.triangle_count
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
