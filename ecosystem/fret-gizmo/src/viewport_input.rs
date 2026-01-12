use fret_core::{Point, Rect, ViewportInputEvent};
use glam::Vec2;

fn rect_size(rect: Rect) -> Vec2 {
    Vec2::new(rect.size.width.0.max(0.0), rect.size.height.0.max(0.0))
}

fn rect_origin(rect: Rect) -> Vec2 {
    Vec2::new(rect.origin.x.0, rect.origin.y.0)
}

fn point_xy(p: Point) -> Vec2 {
    Vec2::new(p.x.0, p.y.0)
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
    let (tw, th) = event.geometry.target_px_size;
    let tw = tw.max(1) as f32;
    let th = th.max(1) as f32;

    let rect = event.geometry.draw_rect_px;
    let size = rect_size(rect);
    if size.x <= 0.0 || size.y <= 0.0 || !size.is_finite() {
        return None;
    }

    let uv = (point_xy(event.cursor_px) - rect_origin(rect)) / size;
    Some(Vec2::new(uv.x * tw, uv.y * th))
}

/// Like [`viewport_input_cursor_target_px`], but clamps the resulting coordinates to the render
/// target bounds.
pub fn viewport_input_cursor_target_px_clamped(event: &ViewportInputEvent) -> Vec2 {
    let (tw, th) = event.geometry.target_px_size;
    let tw = tw.max(1) as f32;
    let th = th.max(1) as f32;

    let Some(p) = viewport_input_cursor_target_px(event) else {
        return Vec2::new(event.target_px.0 as f32, event.target_px.1 as f32);
    };
    Vec2::new(p.x.clamp(0.0, tw), p.y.clamp(0.0, th))
}

#[cfg(test)]
mod tests {
    use fret_core::geometry::{Point, Px, Rect, Size};
    use fret_core::{
        AppWindowId, RenderTargetId, ViewportFit, ViewportInputEvent, ViewportInputGeometry,
        ViewportInputKind,
    };

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
}
