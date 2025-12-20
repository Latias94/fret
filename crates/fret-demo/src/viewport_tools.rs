use fret_core::{AppWindowId, Modifiers, RenderTargetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportToolMode {
    Select,
    Move,
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
    PanOrbit,
    TranslateGizmo,
}

#[derive(Debug, Clone)]
pub enum ViewportInteraction {
    MarqueeSelect(MarqueeSelectInteraction),
    PanOrbit(PanOrbitInteraction),
    TranslateGizmo(TranslateGizmoInteraction),
}

impl ViewportInteraction {
    #[allow(dead_code)]
    pub fn kind(&self) -> ViewportInteractionKind {
        match self {
            ViewportInteraction::MarqueeSelect(_) => ViewportInteractionKind::MarqueeSelect,
            ViewportInteraction::PanOrbit(_) => ViewportInteractionKind::PanOrbit,
            ViewportInteraction::TranslateGizmo(_) => ViewportInteractionKind::TranslateGizmo,
        }
    }

    #[allow(dead_code)]
    pub fn window_target(&self) -> (AppWindowId, RenderTargetId) {
        match self {
            ViewportInteraction::MarqueeSelect(m) => (m.window, m.target),
            ViewportInteraction::PanOrbit(m) => (m.window, m.target),
            ViewportInteraction::TranslateGizmo(m) => (m.window, m.target),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanOrbitKind {
    Orbit,
    Pan,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanOrbitInteraction {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub kind: PanOrbitKind,
    pub start_modifiers: Modifiers,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
    pub dragging: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslateAxisConstraint {
    Free,
    X,
    Y,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslateGizmoInteraction {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_modifiers: Modifiers,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
    pub dragging: bool,
    pub constraint: TranslateAxisConstraint,
    pub targets: Vec<u64>,
    pub start_positions: Vec<(u64, [f32; 3])>,
}
