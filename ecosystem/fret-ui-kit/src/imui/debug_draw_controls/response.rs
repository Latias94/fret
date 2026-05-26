use std::sync::Arc;

use fret_core::Rect;

use crate::imui::ResponseExt;

use super::{DebugDrawCommandSummary, DebugDrawListSummary};

#[derive(Debug, Clone)]
pub struct DebugDrawResponse {
    response: ResponseExt,
    list_summary: DebugDrawListSummary,
    command_summaries: Arc<[DebugDrawCommandSummary]>,
}

impl DebugDrawResponse {
    pub(crate) fn new(
        response: ResponseExt,
        list_summary: DebugDrawListSummary,
        command_summaries: Arc<[DebugDrawCommandSummary]>,
    ) -> Self {
        Self {
            response,
            list_summary,
            command_summaries,
        }
    }

    pub fn response(&self) -> ResponseExt {
        self.response
    }

    pub fn command_summaries(&self) -> &[DebugDrawCommandSummary] {
        &self.command_summaries
    }

    pub fn list_summary(&self) -> DebugDrawListSummary {
        self.list_summary
    }

    pub fn clicked(&self) -> bool {
        self.response.clicked()
    }

    pub fn hovered_like_imgui(&self) -> bool {
        self.response.hovered_like_imgui()
    }

    pub fn rect(&self) -> Option<Rect> {
        self.response.rect()
    }
}
