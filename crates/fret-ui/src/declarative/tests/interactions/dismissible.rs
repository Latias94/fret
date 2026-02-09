use super::*;

#[test]
fn dismissible_layer_pointer_move_observer_does_not_break_click_through() {
    struct CountPointerDown {
        clicks: fret_runtime::Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountPointerDown {
        fn hit_test(&self, bounds: Rect, position: Point) -> bool {
            bounds.contains(position)
        }

        fn event(&mut self, cx: &mut crate::widget::EventCx<'_, H>, event: &fret_core::Event) {
            if matches!(
                event,
                fret_core::Event::Pointer(fret_core::PointerEvent::Down { .. })
            ) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.clicks, |v: &mut u32| *v = v.saturating_add(1));
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let clicks = app.models_mut().insert(0u32);
    let base = ui.create_node(CountPointerDown {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let moves = app.models_mut().insert(0u32);
    let moves_for_hook = moves.clone();
    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-pointer-move-observer",
        move |cx| {
            cx.dismissible_on_pointer_move(Arc::new(move |host, _acx, _mv| {
                let _ = host
                    .models_mut()
                    .update(&moves_for_hook, |v: &mut u32| *v = v.saturating_add(1));
                false
            }));
            Vec::new()
        },
    );
    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);
    ui.set_layer_hit_testable(layer, true);
    ui.set_layer_wants_pointer_move_events(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let p = Point::new(Px(10.0), Px(10.0));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: p,
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&moves), Some(1));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: p,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(
        app.models().get_copied(&clicks),
        Some(1),
        "expected click-through dispatch to reach the underlay"
    );
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_escape() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-escape",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                assert_eq!(req.reason, DismissReason::Escape);
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));

            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _| {
                    vec![cx.text("child")]
                }),
            ]
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Focus a descendant in the overlay so Escape bubbles up to the dismissible layer.
    let focused = ui.children(overlay_root)[0];
    ui.set_focus(Some(focused));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::KeyDown {
            key: fret_core::KeyCode::Escape,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}

#[test]
fn dismissible_on_dismiss_request_hook_runs_on_outside_press_observer() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(200.0), Px(80.0)));
    let mut services = FakeTextService::default();

    let dismissed = app.models_mut().insert(false);

    // Base root provides a hit-test target so the pointer down is "outside" the overlay.
    let base_root = ui.create_node(FillStack);
    ui.set_root(base_root);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "dismissible-hook-outside-press",
        |cx| {
            let dismissed = dismissed.clone();
            cx.dismissible_on_dismiss_request(Arc::new(move |host, _cx, req| {
                match req.reason {
                    DismissReason::OutsidePress { pointer: Some(cx) } => {
                        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
                        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
                        assert_eq!(cx.button, MouseButton::Left);
                        assert_eq!(cx.modifiers, Modifiers::default());
                        assert_eq!(cx.click_count, 1);
                    }
                    other => panic!("expected outside-press dismissal, got {other:?}"),
                }
                let _ = host
                    .models_mut()
                    .update(&dismissed, |v: &mut bool| *v = true);
            }));
            Vec::new()
        },
    );

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer down hits the base root (overlay has no children and is hit-test transparent),
    // so outside-press observer dispatch runs for the overlay root.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(2.0), Px(2.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&dismissed), Some(true));
}
