use super::*;

#[test]
fn pointer_move_observers_are_suppressed_when_pointer_is_captured_by_another_layer() {
    #[derive(Default)]
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
            }
        }
    }

    #[derive(Default)]
    struct CountMoves {
        moves: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for CountMoves {
        fn event(&mut self, _cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
                self.moves.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());

    let base = ui.create_node(CaptureOnDown);
    ui.set_root(base);

    let overlay_moves = Arc::new(AtomicUsize::new(0));
    let overlay_root = ui.create_node(CountMoves {
        moves: overlay_moves.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay_root, false, false);
    ui.set_layer_wants_pointer_move_events(layer, true);
    ui.set_layer_visible(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(20.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(20.0), Px(10.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        overlay_moves.load(Ordering::SeqCst),
        0,
        "expected overlay pointer-move observers to be suppressed while the pointer is captured by the underlay"
    );
}
