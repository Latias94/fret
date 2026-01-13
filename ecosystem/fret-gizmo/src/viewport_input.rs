use fret_core::geometry::{Point, Rect};
use fret_core::{MouseButton, ViewportInputEvent, ViewportInputKind};
use glam::Vec2;

use crate::{GizmoInput, ViewGizmoInput, ViewportRect};

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
    /// Derives a gizmo-friendly input in render-target pixel space (recommended).
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

    /// Derives a gizmo-friendly input in window logical pixel space.
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

    pub fn to_gizmo_input(
        self,
        hovered: bool,
        snap: bool,
        cancel: bool,
        precision: f32,
    ) -> GizmoInput {
        GizmoInput {
            cursor_px: self.cursor_px,
            hovered,
            drag_started: self.drag_started,
            dragging: self.dragging,
            snap,
            cancel,
            precision,
        }
    }

    pub fn to_view_gizmo_input(self, hovered: bool) -> ViewGizmoInput {
        ViewGizmoInput {
            cursor_px: self.cursor_px,
            hovered,
            drag_started: self.drag_started,
            dragging: self.dragging,
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
        assert!((derived.cursor_units_per_screen_px - 1.0).abs() < 1e-6);
        assert!(derived.cursor_over_draw_rect);
    }
}
