//! Tier A viewport tooling helpers.
//!
//! This crate provides policy-light, unit-explicit glue for building editor-style viewport tools
//! (gizmos, selection, camera navigation, debug overlays) on top of `ViewportSurface` and
//! `Effect::ViewportInput` (ADR 0007 / ADR 0147).
//!
//! The goal is to share recurring input mapping logic across ecosystem crates without forcing
//! them to depend on `fret-gizmo`.

use fret_core::geometry::{Point, Rect};
use fret_core::{MouseButton, ViewportInputEvent, ViewportInputKind};
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewportToolId(pub u64);

impl ViewportToolId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ViewportToolPriority(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportToolResult {
    pub handled: bool,
    /// When true, the router should grant this tool exclusive ownership of subsequent pointer
    /// events until the primary button is released (or cancelled).
    pub capture: bool,
}

impl ViewportToolResult {
    pub const fn unhandled() -> Self {
        Self {
            handled: false,
            capture: false,
        }
    }

    pub const fn handled() -> Self {
        Self {
            handled: true,
            capture: false,
        }
    }

    pub const fn handled_and_capture() -> Self {
        Self {
            handled: true,
            capture: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportToolCx<'a> {
    pub event: &'a ViewportInputEvent,
    pub input: ViewportToolInput,
}

/// A viewport tool protocol for editor-style affordances (gizmo, selection, camera navigation).
///
/// This is intentionally policy-light: tool priority and tool activation/capture are decisions
/// made by the host router (see ADR 0168).
pub trait ViewportTool {
    fn id(&self) -> ViewportToolId;

    fn priority(&self) -> ViewportToolPriority {
        ViewportToolPriority::default()
    }

    /// Updates hot/hover affordances (optional).
    fn set_hot(&mut self, _hot: bool) {}

    /// Whether this tool is a hit-test candidate at the current cursor position.
    ///
    /// Routers typically use this to select a single "hot" tool for hover feedback.
    fn hit_test(&mut self, _cx: ViewportToolCx<'_>) -> bool {
        false
    }

    /// Handles an input event. `hot` indicates the current hover winner and `active` indicates
    /// the router has granted this tool exclusive ownership of the interaction.
    fn handle_event(
        &mut self,
        _cx: ViewportToolCx<'_>,
        _hot: bool,
        _active: bool,
    ) -> ViewportToolResult {
        ViewportToolResult::unhandled()
    }

    /// Cancels an in-progress interaction, if any.
    fn cancel(&mut self) {}
}

/// Viewport rectangle in logical or physical pixels (caller-defined).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportRect {
    pub min: Vec2,
    pub size: Vec2,
}

impl ViewportRect {
    pub fn new(min: Vec2, size: Vec2) -> Self {
        Self { min, size }
    }

    pub fn max(self) -> Vec2 {
        self.min + self.size
    }
}

/// A 2D point in viewport coordinates (top-left origin).
pub type ScreenPoint = Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportToolInput {
    /// Viewport rectangle in the same "pixel space" as `cursor_px` (caller-defined).
    pub viewport: ViewportRect,
    /// Cursor position in the same "pixel space" as `viewport` (caller-defined).
    pub cursor_px: Vec2,
    /// Drag edge for the chosen primary button.
    pub drag_started: bool,
    /// Drag state for the chosen primary button.
    pub dragging: bool,
    /// Conversion factor from screen logical pixels to `cursor_px` units.
    ///
    /// - For target-pixel inputs, this is `ViewportInputEvent::target_px_per_screen_px()` (fallback
    ///   to `pixels_per_point`).
    /// - For screen-pixel inputs, this is always `1.0`.
    pub cursor_units_per_screen_px: f32,
    /// Whether the raw cursor is inside the mapped draw rect (window logical px).
    ///
    /// Useful as a default `hovered` gate when the host does not have a higher-level tool
    /// arbitration policy.
    pub cursor_over_draw_rect: bool,
}

impl ViewportToolInput {
    /// Derives a tool input in render-target pixel space (recommended).
    pub fn from_viewport_input_target_px(
        event: &ViewportInputEvent,
        primary_button: MouseButton,
    ) -> Self {
        let (drag_started, dragging) = drag_flags(event.kind, primary_button);
        let (tw, th) = event.geometry.target_px_size;
        let cursor_px = event
            .cursor_target_px_f32()
            .map(|(x, y)| Vec2::new(x, y))
            .unwrap_or_else(|| Vec2::new(event.uv.0 * tw as f32, event.uv.1 * th as f32));
        let cursor_units_per_screen_px = event
            .target_px_per_screen_px()
            .unwrap_or(event.geometry.pixels_per_point.max(1.0e-6));

        Self {
            viewport: ViewportRect::new(Vec2::ZERO, Vec2::new(tw as f32, th as f32)),
            cursor_px,
            drag_started,
            dragging,
            cursor_units_per_screen_px,
            cursor_over_draw_rect: point_in_rect_px(event.cursor_px, event.geometry.draw_rect_px),
        }
    }

    /// Builds a tool input in render-target pixel space when the host already has cursor
    /// coordinates in target pixels (no `ViewportInputEvent` required).
    ///
    /// This is useful for non-UI hosts or for input paths that do not originate from
    /// `Effect::ViewportInput` (e.g. command-driven cancellation).
    pub fn from_target_px_viewport(
        target_px_size: (u32, u32),
        cursor_px: Vec2,
        drag_started: bool,
        dragging: bool,
        cursor_units_per_screen_px: f32,
    ) -> Self {
        let (tw, th) = target_px_size;
        Self {
            viewport: ViewportRect::new(Vec2::ZERO, Vec2::new(tw as f32, th as f32)),
            cursor_px,
            drag_started,
            dragging,
            cursor_units_per_screen_px: if cursor_units_per_screen_px.is_finite()
                && cursor_units_per_screen_px > 0.0
            {
                cursor_units_per_screen_px
            } else {
                1.0
            },
            cursor_over_draw_rect: true,
        }
    }

    /// Derives a tool input in window logical pixel space.
    pub fn from_viewport_input_screen_px(
        event: &ViewportInputEvent,
        primary_button: MouseButton,
    ) -> Self {
        let (drag_started, dragging) = drag_flags(event.kind, primary_button);
        let draw = event.geometry.draw_rect_px;

        Self {
            viewport: ViewportRect::new(rect_min_px(draw), rect_size_px(draw)),
            cursor_px: point_px(event.cursor_px),
            drag_started,
            dragging,
            cursor_units_per_screen_px: 1.0,
            cursor_over_draw_rect: point_in_rect_px(event.cursor_px, draw),
        }
    }
}

fn drag_flags(kind: ViewportInputKind, primary_button: MouseButton) -> (bool, bool) {
    match kind {
        ViewportInputKind::PointerDown { button, .. } if button == primary_button => (true, true),
        ViewportInputKind::PointerMove { buttons, .. } => (
            false,
            match primary_button {
                MouseButton::Left => buttons.left,
                MouseButton::Right => buttons.right,
                MouseButton::Middle => buttons.middle,
                MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => false,
            },
        ),
        ViewportInputKind::PointerUp { button, .. } if button == primary_button => (false, false),
        _ => (false, false),
    }
}

fn point_px(p: Point) -> Vec2 {
    Vec2::new(p.x.0, p.y.0)
}

fn rect_min_px(r: Rect) -> Vec2 {
    Vec2::new(r.origin.x.0, r.origin.y.0)
}

fn rect_size_px(r: Rect) -> Vec2 {
    Vec2::new(r.size.width.0, r.size.height.0)
}

fn point_in_rect_px(p: Point, r: Rect) -> bool {
    let x0 = r.origin.x.0;
    let y0 = r.origin.y.0;
    let x1 = x0 + r.size.width.0;
    let y1 = y0 + r.size.height.0;
    p.x.0 >= x0 && p.y.0 >= y0 && p.x.0 <= x1 && p.y.0 <= y1
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Px, Size};
    use fret_core::{AppWindowId, Modifiers, RenderTargetId, ViewportFit, ViewportInputGeometry};

    fn dummy_event(kind: ViewportInputKind) -> ViewportInputEvent {
        ViewportInputEvent {
            window: AppWindowId::default(),
            target: RenderTargetId::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
            geometry: ViewportInputGeometry {
                content_rect_px: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                draw_rect_px: Rect::new(
                    Point::new(Px(10.0), Px(20.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                target_px_size: (1000, 500),
                fit: ViewportFit::Stretch,
                pixels_per_point: 2.0,
            },
            cursor_px: Point::new(Px(60.0), Px(45.0)),
            uv: (0.25, 0.5),
            target_px: (250, 250),
            kind,
        }
    }

    #[test]
    fn target_px_strategy_derives_viewport_and_cursor() {
        let event = dummy_event(ViewportInputKind::PointerMove {
            buttons: Default::default(),
            modifiers: Modifiers::default(),
        });

        let derived = ViewportToolInput::from_viewport_input_target_px(&event, MouseButton::Left);
        assert_eq!(derived.viewport.min, Vec2::ZERO);
        assert_eq!(derived.viewport.size, Vec2::new(1000.0, 500.0));
        // Draw rect is (10,20) with size (100,50); cursor is (60,45) => normalized (0.5,0.5).
        assert!((derived.cursor_px.x - 500.0).abs() < 1e-3);
        assert!((derived.cursor_px.y - 250.0).abs() < 1e-3);
        assert!((derived.cursor_units_per_screen_px - 10.0).abs() < 1e-6);
        assert!(derived.cursor_over_draw_rect);
    }

    #[test]
    fn target_px_strategy_falls_back_to_uv_when_mapping_unavailable() {
        let mut event = dummy_event(ViewportInputKind::PointerMove {
            buttons: Default::default(),
            modifiers: Modifiers::default(),
        });
        // Make mapping invalid by zeroing out the draw rect size.
        event.geometry.draw_rect_px.size.width = Px(0.0);
        event.geometry.draw_rect_px.size.height = Px(0.0);

        let derived = ViewportToolInput::from_viewport_input_target_px(&event, MouseButton::Left);
        assert!((derived.cursor_px.x - 250.0).abs() < 1e-3);
        assert!((derived.cursor_px.y - 250.0).abs() < 1e-3);
        assert!(!derived.cursor_over_draw_rect);
    }

    #[test]
    fn drag_flags_follow_primary_button() {
        let down = ViewportToolInput::from_viewport_input_target_px(
            &dummy_event(ViewportInputKind::PointerDown {
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
            }),
            MouseButton::Left,
        );
        assert!(down.drag_started);
        assert!(down.dragging);

        let move_dragging = ViewportToolInput::from_viewport_input_target_px(
            &dummy_event(ViewportInputKind::PointerMove {
                buttons: fret_core::MouseButtons {
                    left: true,
                    right: false,
                    middle: false,
                },
                modifiers: Modifiers::default(),
            }),
            MouseButton::Left,
        );
        assert!(!move_dragging.drag_started);
        assert!(move_dragging.dragging);

        let up = ViewportToolInput::from_viewport_input_target_px(
            &dummy_event(ViewportInputKind::PointerUp {
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
            }),
            MouseButton::Left,
        );
        assert!(!up.drag_started);
        assert!(!up.dragging);
    }

    #[test]
    fn screen_px_strategy_derives_draw_rect_space() {
        let event = dummy_event(ViewportInputKind::PointerMove {
            buttons: Default::default(),
            modifiers: Modifiers::default(),
        });

        let derived = ViewportToolInput::from_viewport_input_screen_px(&event, MouseButton::Left);
        assert_eq!(derived.viewport.min, Vec2::new(10.0, 20.0));
        assert_eq!(derived.viewport.size, Vec2::new(100.0, 50.0));
        assert_eq!(derived.cursor_px, Vec2::new(60.0, 45.0));
        assert_eq!(derived.cursor_units_per_screen_px, 1.0);
        assert!(derived.cursor_over_draw_rect);
    }

    #[test]
    fn target_px_viewport_constructor_uses_target_bounds() {
        let derived = ViewportToolInput::from_target_px_viewport(
            (800, 600),
            Vec2::new(1.0, 2.0),
            true,
            true,
            3.0,
        );
        assert_eq!(derived.viewport.min, Vec2::ZERO);
        assert_eq!(derived.viewport.size, Vec2::new(800.0, 600.0));
        assert!(derived.cursor_over_draw_rect);
    }
}
