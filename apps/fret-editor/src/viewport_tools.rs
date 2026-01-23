use fret_core::geometry::{Point, Px, Rect};
use fret_core::{
    AppWindowId, Modifiers, MouseButton, MouseButtons, RenderTargetId, ViewportInputEvent,
    ViewportInputKind,
};

const DEFAULT_GIZMO_CENTER_UV: (f32, f32) = (0.5, 0.5);
const DEFAULT_TRANSLATE_AXIS_LEN_PX: Px = Px(72.0);
const DEFAULT_ROTATE_RADIUS_PX: Px = Px(64.0);
const DEFAULT_GIZMO_PICK_TOLERANCE_PX: Px = Px(6.0);
const DEFAULT_GIZMO_HANDLE_SIZE_PX: Px = Px(10.0);

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
    pub hover_translate: Option<crate::viewport_overlays::ViewportTranslateGizmoPart>,
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

impl ViewportToolManager {
    pub fn handle_viewport_input(&mut self, event: &ViewportInputEvent) -> bool {
        match event.kind {
            ViewportInputKind::PointerDown {
                button, modifiers, ..
            } => self.handle_pointer_down(event, button, modifiers),
            ViewportInputKind::PointerMove { buttons, modifiers } => {
                self.handle_pointer_move(event, buttons, modifiers)
            }
            ViewportInputKind::PointerUp {
                button, modifiers, ..
            } => self.handle_pointer_up(event, button, modifiers),
            ViewportInputKind::PointerCancel { .. } => {
                self.interaction = None;
                self.hover_translate = None;
                self.hover_rotate = None;
                false
            }
            ViewportInputKind::Wheel { .. } => false,
        }
    }

    pub fn overlay(&self) -> crate::viewport_overlays::ViewportOverlay {
        let mut overlay = crate::viewport_overlays::ViewportOverlay {
            marquee: None,
            drag_line: None,
            selection_rect: None,
            translate_gizmo: None,
            rotate_gizmo: None,
            marker: None,
        };

        match self.active {
            ViewportToolMode::Select => {}
            ViewportToolMode::Move => {
                overlay.translate_gizmo = Some(crate::viewport_overlays::ViewportTranslateGizmo {
                    center_uv: DEFAULT_GIZMO_CENTER_UV,
                    axis_len_px: DEFAULT_TRANSLATE_AXIS_LEN_PX,
                    highlight: self.hover_translate,
                });
            }
            ViewportToolMode::Rotate => {
                overlay.rotate_gizmo = Some(crate::viewport_overlays::ViewportRotateGizmo {
                    center_uv: DEFAULT_GIZMO_CENTER_UV,
                    radius_px: DEFAULT_ROTATE_RADIUS_PX,
                    highlight: self.hover_rotate.is_some(),
                });
            }
        }

        let Some(interaction) = self.interaction.as_ref() else {
            return overlay;
        };

        match interaction {
            ViewportInteraction::MarqueeSelect(m) => {
                if m.dragging() {
                    overlay.marquee = Some(crate::viewport_overlays::ViewportMarquee {
                        a_uv: m.start_uv,
                        b_uv: m.current_uv,
                    });
                }
            }
            ViewportInteraction::PanOrbit(_)
            | ViewportInteraction::TranslateGizmo(_)
            | ViewportInteraction::RotateGizmo(_) => {
                let (window, target) = interaction.window_target();
                let _ = (window, target);
            }
        }

        match interaction {
            ViewportInteraction::TranslateGizmo(m) => {
                if m.dragging {
                    overlay.drag_line = Some(crate::viewport_overlays::ViewportDragLine {
                        a_uv: m.start_uv,
                        b_uv: m.current_uv,
                        color: fret_core::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.85,
                        },
                    });
                }
                overlay.translate_gizmo = Some(crate::viewport_overlays::ViewportTranslateGizmo {
                    center_uv: DEFAULT_GIZMO_CENTER_UV,
                    axis_len_px: DEFAULT_TRANSLATE_AXIS_LEN_PX,
                    highlight: match m.constraint {
                        TranslateAxisConstraint::Free => {
                            Some(crate::viewport_overlays::ViewportTranslateGizmoPart::Handle)
                        }
                        TranslateAxisConstraint::X => {
                            Some(crate::viewport_overlays::ViewportTranslateGizmoPart::X)
                        }
                        TranslateAxisConstraint::Y => {
                            Some(crate::viewport_overlays::ViewportTranslateGizmoPart::Y)
                        }
                    },
                });
            }
            ViewportInteraction::RotateGizmo(m) => {
                overlay.rotate_gizmo = Some(crate::viewport_overlays::ViewportRotateGizmo {
                    center_uv: m.center_uv,
                    radius_px: DEFAULT_ROTATE_RADIUS_PX,
                    highlight: true,
                });
                if m.dragging {
                    overlay.drag_line = Some(crate::viewport_overlays::ViewportDragLine {
                        a_uv: m.center_uv,
                        b_uv: m.current_uv,
                        color: fret_core::Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 0.85,
                        },
                    });
                }
            }
            _ => {}
        }

        overlay
    }

    fn handle_pointer_down(
        &mut self,
        event: &ViewportInputEvent,
        button: MouseButton,
        modifiers: Modifiers,
    ) -> bool {
        if self.interaction.is_some() {
            return false;
        }

        match button {
            MouseButton::Left => match self.active {
                ViewportToolMode::Select => {
                    self.interaction = Some(ViewportInteraction::MarqueeSelect(
                        MarqueeSelectInteraction {
                            window: event.window,
                            target: event.target,
                            start_modifiers: modifiers,
                            start_cursor_px: event.cursor_px,
                            current_cursor_px: event.cursor_px,
                            start_uv: event.uv,
                            current_uv: event.uv,
                            start_target_px: event.target_px,
                            current_target_px: event.target_px,
                        },
                    ));
                    true
                }
                ViewportToolMode::Move => {
                    let Some(part) = self.hover_translate else {
                        return false;
                    };
                    let constraint = match part {
                        crate::viewport_overlays::ViewportTranslateGizmoPart::X => {
                            TranslateAxisConstraint::X
                        }
                        crate::viewport_overlays::ViewportTranslateGizmoPart::Y => {
                            TranslateAxisConstraint::Y
                        }
                        crate::viewport_overlays::ViewportTranslateGizmoPart::Handle => {
                            TranslateAxisConstraint::Free
                        }
                    };
                    self.interaction = Some(ViewportInteraction::TranslateGizmo(
                        TranslateGizmoInteraction {
                            window: event.window,
                            target: event.target,
                            start_modifiers: modifiers,
                            start_cursor_px: event.cursor_px,
                            current_cursor_px: event.cursor_px,
                            start_uv: event.uv,
                            current_uv: event.uv,
                            start_target_px: event.target_px,
                            current_target_px: event.target_px,
                            dragging: false,
                            constraint,
                            targets: Vec::new(),
                            start_positions: Vec::new(),
                        },
                    ));
                    true
                }
                ViewportToolMode::Rotate => {
                    if self.hover_rotate.is_none() {
                        return false;
                    }

                    let (tw, th) = event.geometry.target_px_size;
                    let (u, v) = DEFAULT_GIZMO_CENTER_UV;
                    let center_target_px = (u * tw.max(1) as f32, v * th.max(1) as f32);
                    let center_screen_px =
                        Self::screen_point_from_uv(event.geometry.draw_rect_px, (u, v));
                    let (use_target_px, start_angle_rad) =
                        if let Some((x, y)) = event.cursor_target_px_f32() {
                            (true, Self::angle_rad((x, y), center_target_px))
                        } else {
                            (
                                false,
                                Self::angle_rad(
                                    (event.cursor_px.x.0, event.cursor_px.y.0),
                                    (center_screen_px.x.0, center_screen_px.y.0),
                                ),
                            )
                        };

                    self.interaction =
                        Some(ViewportInteraction::RotateGizmo(RotateGizmoInteraction {
                            window: event.window,
                            target: event.target,
                            start_modifiers: modifiers,
                            start_cursor_px: event.cursor_px,
                            current_cursor_px: event.cursor_px,
                            center_uv: DEFAULT_GIZMO_CENTER_UV,
                            start_uv: event.uv,
                            current_uv: event.uv,
                            start_target_px: event.target_px,
                            current_target_px: event.target_px,
                            center_target_px,
                            start_angle_rad,
                            use_target_px,
                            dragging: false,
                            targets: Vec::new(),
                            start_rotations: Vec::new(),
                        }));
                    true
                }
            },
            MouseButton::Right | MouseButton::Middle => {
                let kind = if button == MouseButton::Middle {
                    PanOrbitKind::Pan
                } else {
                    PanOrbitKind::Orbit
                };
                self.interaction = Some(ViewportInteraction::PanOrbit(PanOrbitInteraction {
                    window: event.window,
                    target: event.target,
                    kind,
                    button,
                    start_modifiers: modifiers,
                    start_cursor_px: event.cursor_px,
                    last_cursor_px: event.cursor_px,
                    current_cursor_px: event.cursor_px,
                    start_uv: event.uv,
                    last_uv: event.uv,
                    current_uv: event.uv,
                    start_target_px: event.target_px,
                    last_target_px: event.target_px,
                    current_target_px: event.target_px,
                    dragging: false,
                }));
                true
            }
            MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
        }
    }

    fn handle_pointer_move(
        &mut self,
        event: &ViewportInputEvent,
        buttons: MouseButtons,
        _modifiers: Modifiers,
    ) -> bool {
        if self.interaction.is_none() {
            let prev_translate = self.hover_translate;
            let prev_rotate = self.hover_rotate;

            self.hover_translate = None;
            self.hover_rotate = None;
            match self.active {
                ViewportToolMode::Select => {}
                ViewportToolMode::Move => {
                    self.hover_translate = Self::hit_test_translate_gizmo(event);
                }
                ViewportToolMode::Rotate => {
                    if Self::hit_test_rotate_gizmo(event) {
                        self.hover_rotate = Some((event.window, event.target));
                    }
                }
            }

            let _ = buttons;
            return self.hover_translate != prev_translate || self.hover_rotate != prev_rotate;
        }

        let Some(interaction) = self.interaction.as_mut() else {
            return false;
        };

        if interaction.window_target() != (event.window, event.target) {
            return false;
        }

        match interaction {
            ViewportInteraction::MarqueeSelect(m) => {
                m.current_cursor_px = event.cursor_px;
                m.current_uv = event.uv;
                m.current_target_px = event.target_px;
                true
            }
            ViewportInteraction::PanOrbit(m) => {
                m.last_cursor_px = m.current_cursor_px;
                m.last_uv = m.current_uv;
                m.last_target_px = m.current_target_px;

                m.current_cursor_px = event.cursor_px;
                m.current_uv = event.uv;
                m.current_target_px = event.target_px;
                m.update_dragging_flag();
                true
            }
            ViewportInteraction::TranslateGizmo(m) => {
                m.current_cursor_px = event.cursor_px;
                m.current_uv = event.uv;
                m.current_target_px = event.target_px;
                if !m.dragging {
                    m.dragging = crossed_screen_drag_threshold(
                        m.start_cursor_px,
                        m.current_cursor_px,
                        VIEWPORT_TOOL_DRAG_THRESHOLD_PX,
                    );
                }
                true
            }
            ViewportInteraction::RotateGizmo(m) => {
                m.current_cursor_px = event.cursor_px;
                m.current_uv = event.uv;
                m.current_target_px = event.target_px;
                if !m.dragging {
                    m.dragging = crossed_screen_drag_threshold(
                        m.start_cursor_px,
                        m.current_cursor_px,
                        VIEWPORT_TOOL_DRAG_THRESHOLD_PX,
                    );
                }
                true
            }
        }
    }

    fn handle_pointer_up(
        &mut self,
        event: &ViewportInputEvent,
        button: MouseButton,
        _modifiers: Modifiers,
    ) -> bool {
        let Some(interaction) = self.interaction.as_ref() else {
            return false;
        };

        if interaction.window_target() != (event.window, event.target) {
            return false;
        }

        let end = match interaction {
            ViewportInteraction::MarqueeSelect(_) => button == MouseButton::Left,
            ViewportInteraction::PanOrbit(m) => button == m.button,
            ViewportInteraction::TranslateGizmo(_) => button == MouseButton::Left,
            ViewportInteraction::RotateGizmo(_) => button == MouseButton::Left,
        };
        if !end {
            return false;
        }

        self.interaction = None;
        true
    }

    fn screen_point_from_uv(draw_rect: Rect, uv: (f32, f32)) -> Point {
        let (u, v) = uv;
        let x = draw_rect.origin.x.0 + draw_rect.size.width.0 * u;
        let y = draw_rect.origin.y.0 + draw_rect.size.height.0 * v;
        Point::new(Px(x), Px(y))
    }

    fn angle_rad(cursor: (f32, f32), center: (f32, f32)) -> f32 {
        let dx = cursor.0 - center.0;
        let dy = cursor.1 - center.1;
        dy.atan2(dx)
    }

    fn hit_test_translate_gizmo(
        event: &ViewportInputEvent,
    ) -> Option<crate::viewport_overlays::ViewportTranslateGizmoPart> {
        let draw_rect = event.geometry.draw_rect_px;
        if draw_rect.size.width.0 <= 0.0 || draw_rect.size.height.0 <= 0.0 {
            return None;
        }

        let center = Self::screen_point_from_uv(draw_rect, DEFAULT_GIZMO_CENTER_UV);
        let cursor = event.cursor_px;
        let dx = cursor.x.0 - center.x.0;
        let dy = cursor.y.0 - center.y.0;

        let t = DEFAULT_GIZMO_PICK_TOLERANCE_PX.0.max(0.0);
        let half_handle = DEFAULT_GIZMO_HANDLE_SIZE_PX.0 * 0.5 + t;
        if dx.abs() <= half_handle && dy.abs() <= half_handle {
            return Some(crate::viewport_overlays::ViewportTranslateGizmoPart::Handle);
        }

        let axis_len = DEFAULT_TRANSLATE_AXIS_LEN_PX.0.max(0.0);
        let axis_half_thickness = DEFAULT_GIZMO_PICK_TOLERANCE_PX.0.max(1.0);

        // X axis: to the right.
        if dx >= 0.0 && dx <= axis_len && dy.abs() <= axis_half_thickness {
            return Some(crate::viewport_overlays::ViewportTranslateGizmoPart::X);
        }
        // Y axis: up.
        if dy <= 0.0 && dy.abs() <= axis_len && dx.abs() <= axis_half_thickness {
            return Some(crate::viewport_overlays::ViewportTranslateGizmoPart::Y);
        }

        None
    }

    fn hit_test_rotate_gizmo(event: &ViewportInputEvent) -> bool {
        let draw_rect = event.geometry.draw_rect_px;
        if draw_rect.size.width.0 <= 0.0 || draw_rect.size.height.0 <= 0.0 {
            return false;
        }

        let center = Self::screen_point_from_uv(draw_rect, DEFAULT_GIZMO_CENTER_UV);
        let cursor = event.cursor_px;
        let dx = cursor.x.0 - center.x.0;
        let dy = cursor.y.0 - center.y.0;
        let dist = (dx * dx + dy * dy).sqrt();

        let r = DEFAULT_ROTATE_RADIUS_PX.0.max(0.0);
        let tol = DEFAULT_GIZMO_PICK_TOLERANCE_PX.0.max(0.0);
        dist.is_finite() && (dist - r).abs() <= tol
    }
}
