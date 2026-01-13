use fret_core::ViewportInputEvent;
use glam::Vec2;

/// Computes the scale factor from screen-space logical pixels to render-target pixels.
///
/// Prefer `ViewportInputEvent::target_px_per_screen_px()` from `fret-core`.
pub fn viewport_input_target_px_per_screen_px(event: &ViewportInputEvent) -> Option<f32> {
    event.target_px_per_screen_px()
}

/// Computes the cursor position in the viewport render target's pixel space.
///
/// - Input `event.cursor_px` is in window-local logical pixels (ADR 0017).
/// - The mapping uses `event.geometry.draw_rect_px` (logical pixels) as the area that corresponds
///   to the full render target.
/// - Output is expressed in physical target pixels (`event.geometry.target_px_size`).
///
/// This is intended to be used by engine/editor tooling during drags because it is derived from
/// the raw cursor position (unclamped), unlike `event.uv`/`event.target_px` which may be clamped
/// when pointer capture is active.
pub fn viewport_input_cursor_target_px(event: &ViewportInputEvent) -> Option<Vec2> {
    event.cursor_target_px_f32().map(|(x, y)| Vec2::new(x, y))
}

/// Like [`viewport_input_cursor_target_px`], but clamps the resulting coordinates to the render
/// target bounds.
pub fn viewport_input_cursor_target_px_clamped(event: &ViewportInputEvent) -> Vec2 {
    let (x, y) = event.cursor_target_px_f32_clamped();
    Vec2::new(x, y)
}

#[cfg(test)]
mod tests {
    use fret_core::geometry::{Point, Px, Rect, Size};
    use fret_core::{
        AppWindowId, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputGeometry,
        ViewportInputKind,
    };

    use super::viewport_input_target_px_per_screen_px;
    use super::{viewport_input_cursor_target_px, viewport_input_cursor_target_px_clamped};

    fn dummy_event(cursor: Point) -> ViewportInputEvent {
        ViewportInputEvent {
            window: AppWindowId::default(),
            target: RenderTargetId::default(),
            geometry: ViewportInputGeometry {
                content_rect_px: Rect::new(
                    Point::new(Px(0.0), Px(0.0)),
                    Size::new(Px(200.0), Px(100.0)),
                ),
                draw_rect_px: Rect::new(
                    Point::new(Px(50.0), Px(25.0)),
                    Size::new(Px(100.0), Px(50.0)),
                ),
                target_px_size: (1000, 500),
                fit: ViewportFit::Contain,
                pixels_per_point: 2.0,
            },
            cursor_px: cursor,
            uv: (0.0, 0.0),
            target_px: (0, 0),
            kind: ViewportInputKind::PointerMove {
                buttons: fret_core::MouseButtons::default(),
                modifiers: fret_core::Modifiers::default(),
            },
        }
    }

    #[test]
    fn cursor_target_px_maps_draw_rect_origin_to_zero() {
        let event = dummy_event(Point::new(Px(50.0), Px(25.0)));
        let p = viewport_input_cursor_target_px(&event).unwrap();
        assert!((p - glam::Vec2::ZERO).length() < 1e-3);
    }

    #[test]
    fn cursor_target_px_maps_draw_rect_max_to_target_size() {
        let event = dummy_event(Point::new(Px(150.0), Px(75.0)));
        let p = viewport_input_cursor_target_px(&event).unwrap();
        assert!((p - glam::Vec2::new(1000.0, 500.0)).length() < 1e-3);
    }

    #[test]
    fn cursor_target_px_clamped_caps_outside_values() {
        let event = dummy_event(Point::new(Px(200.0), Px(125.0)));
        let p = viewport_input_cursor_target_px_clamped(&event);
        assert_eq!(p, glam::Vec2::new(1000.0, 500.0));
    }

    #[test]
    fn target_px_per_screen_px_matches_draw_rect_mapping() {
        let event = dummy_event(Point::new(Px(100.0), Px(50.0)));
        let scale = viewport_input_target_px_per_screen_px(&event).unwrap();
        // draw_rect is 100x50 logical px and target is 1000x500 physical px, so 10 target px per
        // screen px.
        assert!((scale - 10.0).abs() < 1e-3);
    }
}
