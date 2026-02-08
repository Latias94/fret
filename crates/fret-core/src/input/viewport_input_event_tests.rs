use super::*;
use crate::geometry::{Point, Px, Rect, Size};
use crate::{AppWindowId, PointerId, RenderTargetId, ViewportFit};

fn dummy_event(cursor: Point) -> ViewportInputEvent {
    ViewportInputEvent {
        window: AppWindowId::default(),
        target: RenderTargetId::default(),
        pointer_id: PointerId(0),
        pointer_type: PointerType::Mouse,
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
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
        },
    }
}

#[test]
fn target_px_per_screen_px_matches_draw_rect_mapping() {
    let event = dummy_event(Point::new(Px(0.0), Px(0.0)));
    let scale = event.target_px_per_screen_px().unwrap();
    assert!((scale - 10.0).abs() < 1e-3);
}

#[test]
fn cursor_target_px_maps_draw_rect_origin_to_zero() {
    let event = dummy_event(Point::new(Px(50.0), Px(25.0)));
    let (x, y) = event.cursor_target_px_f32().unwrap();
    assert!(((x - 0.0).powi(2) + (y - 0.0).powi(2)).sqrt() < 1e-3);
}

#[test]
fn cursor_target_px_maps_draw_rect_max_to_target_size() {
    let event = dummy_event(Point::new(Px(150.0), Px(75.0)));
    let (x, y) = event.cursor_target_px_f32().unwrap();
    assert!(((x - 1000.0).powi(2) + (y - 500.0).powi(2)).sqrt() < 1e-3);
}

#[test]
fn cursor_target_px_clamped_caps_outside_values() {
    let event = dummy_event(Point::new(Px(200.0), Px(125.0)));
    let (x, y) = event.cursor_target_px_f32_clamped();
    assert_eq!((x, y), (1000.0, 500.0));
}
