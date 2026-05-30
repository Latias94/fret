use super::DebugDrawCommand;

mod channels;
mod clips;
mod core;
mod images;
mod summaries;
mod svg_text;

#[derive(Debug, Clone)]
pub struct ImUiDebugDrawList {
    pub(in crate::imui::debug_draw_controls) commands: Vec<DebugDrawCommand>,
    pub(in crate::imui::debug_draw_controls) channel_split: Option<DebugDrawChannelSplit>,
}

#[derive(Debug, Clone)]
pub(in crate::imui::debug_draw_controls) struct DebugDrawChannelSplit {
    channels: Vec<Vec<DebugDrawCommand>>,
    current: usize,
}
