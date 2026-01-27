use super::*;

#[test]
fn dock_drag_suppresses_pointer_capture_requests_from_non_anchor_nodes() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CaptureOnDown);
    ui.set_root(base);

    let anchor = ui.create_node(CaptureOnDown);
    crate::internal_drag::set_route(&mut app, window, fret_runtime::DRAG_KIND_DOCK_PANEL, anchor);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
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

    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        None,
        "expected non-anchor capture requests to be ignored during dock drag"
    );
}

#[test]
fn dock_drag_allows_pointer_capture_for_anchor_node() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let anchor = ui.create_node(CaptureOnDown);
    ui.set_root(anchor);

    crate::internal_drag::set_route(&mut app, window, fret_runtime::DRAG_KIND_DOCK_PANEL, anchor);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
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

    assert_eq!(
        ui.captured_for(fret_core::PointerId(0)),
        Some(anchor),
        "expected dock-drag anchor to be allowed to capture during dock drag"
    );
}

#[test]
fn escape_cancels_dock_drag_and_does_not_dismiss_overlays() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let dismissed = app.models_mut().insert(false);

    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(TestStack::default());
    ui.set_root(base);

    let overlay = ui.create_node(TestStack::default());
    let overlay_element = crate::GlobalElementId(0xdead_beef);
    ui.set_node_element(overlay, Some(overlay_element));
    let _layer = ui.push_overlay_root_ex(overlay, false, true);

    crate::elements::with_element_state(
        &mut app,
        window,
        overlay_element,
        crate::action::DismissibleActionHooks::default,
        |hooks| {
            let dismissed = dismissed.clone();
            hooks.on_dismiss_request = Some(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, crate::action::DismissReason::Escape);
                let _ = host.models_mut().update(&dismissed, |v| *v = true);
            }));
        },
    );

    // Establish a drag route anchor so capture rules can resolve it deterministically.
    let anchor = ui.create_node(TestStack::default());
    crate::internal_drag::set_route(&mut app, window, fret_runtime::DRAG_KIND_DOCK_PANEL, anchor);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: fret_core::Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.drag(fret_core::PointerId(7)).is_none(),
        "expected Escape to cancel the dock drag session"
    );
    assert_eq!(
        app.models().get_copied(&dismissed),
        Some(false),
        "expected Escape to not dismiss overlays during dock drag"
    );
}
