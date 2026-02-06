use super::*;

#[test]
fn hit_test_path_cache_reuse_policy_covers_high_frequency_events() {
    let pointer_id = fret_core::PointerId(0);
    let position = Point::new(Px(10.0), Px(20.0));
    let modifiers = fret_core::Modifiers::default();
    let pointer_type = fret_core::PointerType::Mouse;

    assert!(event_allows_hit_test_path_cache_reuse(&Event::Pointer(
        PointerEvent::Move {
            pointer_id,
            position,
            buttons: fret_core::MouseButtons::default(),
            modifiers,
            pointer_type,
        }
    )));
    assert!(event_allows_hit_test_path_cache_reuse(&Event::Pointer(
        PointerEvent::Wheel {
            pointer_id,
            position,
            delta: Point::new(Px(0.0), Px(120.0)),
            modifiers,
            pointer_type,
        }
    )));
    assert!(event_allows_hit_test_path_cache_reuse(&Event::Pointer(
        PointerEvent::PinchGesture {
            pointer_id,
            position,
            delta: 1.0,
            modifiers,
            pointer_type,
        }
    )));

    assert!(!event_allows_hit_test_path_cache_reuse(&Event::Pointer(
        PointerEvent::Down {
            pointer_id,
            position,
            button: fret_core::MouseButton::Left,
            modifiers,
            click_count: 1,
            pointer_type,
        }
    )));
    assert!(!event_allows_hit_test_path_cache_reuse(&Event::Pointer(
        PointerEvent::Up {
            pointer_id,
            position,
            button: fret_core::MouseButton::Left,
            modifiers,
            is_click: true,
            click_count: 1,
            pointer_type,
        }
    )));

    assert!(event_allows_hit_test_path_cache_reuse(
        &Event::ExternalDrag(fret_core::ExternalDragEvent {
            position,
            kind: fret_core::ExternalDragKind::Leave,
        })
    ));
    assert!(event_allows_hit_test_path_cache_reuse(
        &Event::InternalDrag(fret_core::InternalDragEvent {
            pointer_id,
            position,
            kind: fret_core::InternalDragKind::Over,
            modifiers,
        })
    ));
}
