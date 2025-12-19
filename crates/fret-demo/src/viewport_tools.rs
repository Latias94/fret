use fret_core::{AppWindowId, Modifiers, RenderTargetId};

#[derive(Debug, Default, Clone)]
pub struct ViewportToolManager {
    pub marquee: Option<ViewportMarqueeState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarqueeState {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_modifiers: Modifiers,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
}
