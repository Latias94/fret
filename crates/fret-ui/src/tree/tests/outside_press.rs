use super::*;

#[test]
fn outside_press_observer_must_not_capture_pointer_or_break_click_through() {
    struct CaptureOnPointerDownOutside;

    impl<H: UiHost> Widget<H> for CaptureOnPointerDownOutside {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(CaptureOnPointerDownOutside);
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

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
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
        }),
    );

    let value = app.models().get_copied(&clicks).unwrap_or(0);
    assert_eq!(
        value, 1,
        "expected click-through dispatch to reach underlay"
    );
    assert_eq!(
        ui.captured(),
        None,
        "observer pass must not capture pointer"
    );
}

#[test]
fn outside_press_observer_dispatch_sets_input_context_phase() {
    struct RecordObserverPhase {
        phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverPhase {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.phase, |v: &mut fret_runtime::InputDispatchPhase| {
                        *v = cx.input_ctx.dispatch_phase
                    });
            }
        }
    }

    struct RecordNormalPhase {
        phase: fret_runtime::Model<fret_runtime::InputDispatchPhase>,
    }

    impl<H: UiHost> Widget<H> for RecordNormalPhase {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.phase, |v: &mut fret_runtime::InputDispatchPhase| {
                        *v = cx.input_ctx.dispatch_phase
                    });
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let observer_phase = app
        .models_mut()
        .insert(fret_runtime::InputDispatchPhase::Normal);
    let normal_phase = app
        .models_mut()
        .insert(fret_runtime::InputDispatchPhase::Observer);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(RecordNormalPhase {
        phase: normal_phase.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverPhase {
        phase: observer_phase.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

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
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_phase),
        Some(fret_runtime::InputDispatchPhase::Observer),
        "observer pass should tag InputContext as Observer"
    );
    assert_eq!(
        app.models().get_copied(&normal_phase),
        Some(fret_runtime::InputDispatchPhase::Normal),
        "normal hit-tested dispatch should tag InputContext as Normal"
    );
}
