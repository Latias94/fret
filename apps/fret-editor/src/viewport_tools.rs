use fret_core::geometry::{Point, Px};
use fret_core::{AppWindowId, Modifiers, MouseButton, RenderTargetId, ViewportInputEvent};

/// Default drag threshold for viewport tools (screen-space logical pixels).
///
/// This intentionally mirrors ImGui's default drag threshold (~6px).
pub const VIEWPORT_TOOL_DRAG_THRESHOLD_PX: Px = Px(6.0);

/// Default marquee-start threshold for viewport tools (screen-space logical pixels).
pub const VIEWPORT_MARQUEE_DRAG_THRESHOLD_PX: Px = Px(4.0);

pub fn screen_drag_distance_sq(start: Point, current: Point) -> f32 {
    let dx = current.x.0 - start.x.0;
    let dy = current.y.0 - start.y.0;
    dx * dx + dy * dy
}

pub fn crossed_screen_drag_threshold(start: Point, current: Point, threshold: Px) -> bool {
    if threshold.0 <= 0.0 {
        return true;
    }
    screen_drag_distance_sq(start, current) >= threshold.0 * threshold.0
}

/// Helper for editor tooling that operates in render-target pixel space.
///
/// Prefer `ViewportInputEvent::cursor_target_px_f32()` directly in new code.
pub fn event_cursor_target_px_f32(event: &ViewportInputEvent) -> Option<(f32, f32)> {
    event.cursor_target_px_f32()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportToolMode {
    #[default]
    Select,
    Move,
    Rotate,
}

#[derive(Debug, Clone, Default)]
pub struct ViewportToolManager {
    pub active: ViewportToolMode,
    pub interaction: Option<ViewportInteraction>,
    pub hover_rotate: Option<(AppWindowId, RenderTargetId)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewportInteractionKind {
    MarqueeSelect,
    PanOrbit,
    TranslateGizmo,
    RotateGizmo,
}

#[derive(Debug, Clone)]
pub enum ViewportInteraction {
    MarqueeSelect(MarqueeSelectInteraction),
    PanOrbit(PanOrbitInteraction),
    TranslateGizmo(TranslateGizmoInteraction),
    RotateGizmo(RotateGizmoInteraction),
}

impl ViewportInteraction {
    #[allow(dead_code)]
    pub fn kind(&self) -> ViewportInteractionKind {
        match self {
            ViewportInteraction::MarqueeSelect(_) => ViewportInteractionKind::MarqueeSelect,
            ViewportInteraction::PanOrbit(_) => ViewportInteractionKind::PanOrbit,
            ViewportInteraction::TranslateGizmo(_) => ViewportInteractionKind::TranslateGizmo,
            ViewportInteraction::RotateGizmo(_) => ViewportInteractionKind::RotateGizmo,
        }
    }

    #[allow(dead_code)]
    pub fn window_target(&self) -> (AppWindowId, RenderTargetId) {
        match self {
            ViewportInteraction::MarqueeSelect(m) => (m.window, m.target),
            ViewportInteraction::PanOrbit(m) => (m.window, m.target),
            ViewportInteraction::TranslateGizmo(m) => (m.window, m.target),
            ViewportInteraction::RotateGizmo(m) => (m.window, m.target),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarqueeSelectInteraction {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_modifiers: Modifiers,
    /// Start cursor position in window-local logical pixels (screen px).
    pub start_cursor_px: Point,
    /// Current cursor position in window-local logical pixels (screen px).
    pub current_cursor_px: Point,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    /// Cursor position in render-target pixels (integer, typically clamped while captured).
    pub start_target_px: (u32, u32),
    /// Cursor position in render-target pixels (integer, typically clamped while captured).
    pub current_target_px: (u32, u32),
}

impl MarqueeSelectInteraction {
    #[allow(dead_code)]
    pub fn dragging(&self) -> bool {
        crossed_screen_drag_threshold(
            self.start_cursor_px,
            self.current_cursor_px,
            VIEWPORT_MARQUEE_DRAG_THRESHOLD_PX,
        )
    }
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
    pub button: MouseButton,
    pub start_modifiers: Modifiers,
    /// Start cursor position in window-local logical pixels (screen px).
    pub start_cursor_px: Point,
    /// Last cursor position in window-local logical pixels (screen px).
    pub last_cursor_px: Point,
    /// Current cursor position in window-local logical pixels (screen px).
    pub current_cursor_px: Point,
    pub start_uv: (f32, f32),
    pub last_uv: (f32, f32),
    pub current_uv: (f32, f32),
    /// Cursor position in render-target pixels (integer, typically clamped while captured).
    pub start_target_px: (u32, u32),
    /// Cursor position in render-target pixels (integer, typically clamped while captured).
    pub last_target_px: (u32, u32),
    /// Cursor position in render-target pixels (integer, typically clamped while captured).
    pub current_target_px: (u32, u32),
    pub dragging: bool,
}

impl PanOrbitInteraction {
    #[allow(dead_code)]
    pub fn update_dragging_flag(&mut self) {
        if self.dragging {
            return;
        }
        self.dragging = crossed_screen_drag_threshold(
            self.start_cursor_px,
            self.current_cursor_px,
            VIEWPORT_TOOL_DRAG_THRESHOLD_PX,
        );
    }
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
    /// Start cursor position in window-local logical pixels (screen px).
    pub start_cursor_px: Point,
    /// Current cursor position in window-local logical pixels (screen px).
    pub current_cursor_px: Point,
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
    pub dragging: bool,
    pub constraint: TranslateAxisConstraint,
    pub targets: Vec<u64>,
    pub start_positions: Vec<(u64, [f32; 3])>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RotateGizmoInteraction {
    pub window: AppWindowId,
    pub target: RenderTargetId,
    pub start_modifiers: Modifiers,
    /// Start cursor position in window-local logical pixels (screen px).
    pub start_cursor_px: Point,
    /// Current cursor position in window-local logical pixels (screen px).
    pub current_cursor_px: Point,
    pub center_uv: (f32, f32),
    pub start_uv: (f32, f32),
    pub current_uv: (f32, f32),
    pub start_target_px: (u32, u32),
    pub current_target_px: (u32, u32),
    pub center_target_px: (f32, f32),
    pub start_angle_rad: f32,
    pub use_target_px: bool,
    pub dragging: bool,
    pub targets: Vec<u64>,
    pub start_rotations: Vec<(u64, f32)>,
}
