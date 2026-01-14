use super::*;

#[test]
fn pointer_move_is_forwarded_to_previous_hover_target() {
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

    let left_moves = Arc::new(AtomicUsize::new(0));
    let right_moves = Arc::new(AtomicUsize::new(0));

    let left = ui.create_node(CountMoves {
        moves: left_moves.clone(),
    });
    let right = ui.create_node(CountMoves {
        moves: right_moves.clone(),
    });

    let root = crate::FixedSplit::create_node_with_children(
        &mut ui,
        crate::FixedSplit::horizontal(0.5),
        left,
        right,
    );
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(20.0)));
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let move_left = Event::Pointer(PointerEvent::Move {
        position: Point::new(Px(10.0), Px(10.0)),
        buttons: fret_core::MouseButtons::default(),
        modifiers: fret_core::Modifiers::default(),
        pointer_id: fret_core::PointerId(0),
        pointer_type: fret_core::PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_left);
    assert_eq!(left_moves.load(Ordering::SeqCst), 1);
    assert_eq!(right_moves.load(Ordering::SeqCst), 0);

    let move_right = Event::Pointer(PointerEvent::Move {
        position: Point::new(Px(90.0), Px(10.0)),
        buttons: fret_core::MouseButtons::default(),
        modifiers: fret_core::Modifiers::default(),
        pointer_id: fret_core::PointerId(0),
        pointer_type: fret_core::PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_right);

    // The right node gets the normal dispatch, and the left node gets an observer dispatch so it
    // can clear hover state when the pointer crosses between siblings.
    assert_eq!(left_moves.load(Ordering::SeqCst), 2);
    assert_eq!(right_moves.load(Ordering::SeqCst), 1);

    let move_right_again = Event::Pointer(PointerEvent::Move {
        position: Point::new(Px(80.0), Px(10.0)),
        buttons: fret_core::MouseButtons::default(),
        modifiers: fret_core::Modifiers::default(),
        pointer_id: fret_core::PointerId(0),
        pointer_type: fret_core::PointerType::Mouse,
    });
    ui.dispatch_event(&mut app, &mut services, &move_right_again);
    assert_eq!(left_moves.load(Ordering::SeqCst), 2);
    assert_eq!(right_moves.load(Ordering::SeqCst), 2);
}
