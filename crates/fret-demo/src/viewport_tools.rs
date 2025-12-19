use fret_core::{AppWindowId, RenderTargetId};

#[derive(Debug, Default, Clone)]
pub struct ViewportToolManager {
    pub marquee: Option<ViewportMarqueeState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportMarqueeState {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
}
