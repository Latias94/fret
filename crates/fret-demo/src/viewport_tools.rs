use fret_core::{AppWindowId, Modifiers, RenderTargetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportToolMode {
    Select,
}

impl Default for ViewportToolMode {
    fn default() -> Self {
        Self::Select
    }
}

#[derive(Debug, Clone)]
pub struct ViewportToolManager {
    pub active: ViewportToolMode,
    pub interaction: Option<ViewportInteraction>,
}

impl Default for ViewportToolManager {
    fn default() -> Self {
        Self {
            active: ViewportToolMode::default(),
            interaction: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportInteractionKind {
    MarqueeSelect,
}

#[derive(Debug, Clone)]
pub enum ViewportInteraction {
    MarqueeSelect(MarqueeSelectInteraction),
}

impl ViewportInteraction {
    #[allow(dead_code)]
    pub fn kind(&self) -> ViewportInteractionKind {
        match self {
            ViewportInteraction::MarqueeSelect(_) => ViewportInteractionKind::MarqueeSelect,
        }
    }

    #[allow(dead_code)]
    pub fn window_target(&self) -> (AppWindowId, RenderTargetId) {
        match self {
            ViewportInteraction::MarqueeSelect(m) => (m.window, m.target),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarqueeSelectInteraction {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_modifiers: Modifiers,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
}
