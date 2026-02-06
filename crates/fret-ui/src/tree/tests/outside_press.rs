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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
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
fn outside_press_observer_respects_overlay_render_transform() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }

        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            let center = Point::new(Px(5.0), Px(5.0));
            let rotate = Transform2D::rotation_about_degrees(90.0, center);
            let translate = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            Some(translate * rotate)
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct CountNormalDown {
        normal: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountNormalDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.normal, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let overlay_down = app.models_mut().insert(0u32);
    let observer_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CountNormalDown {
        normal: underlay_down.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let overlay_leaf = ui.create_node(CountNormalDown {
        normal: overlay_down.clone(),
    });
    ui.add_child(overlay_root, overlay_leaf);
    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // This window-space point maps to local (6, 5) inside the overlay leaf after rotation+translation.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(45.0), Px(6.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(0),
        "expected inside click to not trigger observer outside-press dispatch"
    );
    assert_eq!(
        app.models().get_copied(&overlay_down),
        Some(1),
        "expected overlay leaf to receive normal pointer down"
    );
    assert_eq!(
        app.models().get_copied(&underlay_down),
        Some(0),
        "expected underlay to not receive pointer down when overlay handles it"
    );

    // Clicking outside the overlay should trigger observer dispatch, but still reach the underlay (click-through).
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
        app.models().get_copied(&observer_down),
        Some(1),
        "expected outside click to trigger observer outside-press dispatch"
    );
    assert_eq!(app.models().get_copied(&underlay_down), Some(1));
}

#[test]
fn outside_press_observer_is_delayed_for_touch_until_pointer_up() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);
    let observer = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer.clone(),
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
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert_eq!(app.models().get_copied(&observer), Some(0));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert_eq!(app.models().get_copied(&observer), Some(1));
    assert_eq!(app.models().get_copied(&clicks), Some(1));
}

#[test]
fn dock_drag_suppresses_outside_press_observer_dispatch_window_globally() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    struct CountNormalDown {
        normal: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountNormalDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.normal, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let observer_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CountNormalDown {
        normal: underlay_down.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Baseline: without dock drag, outside-press observer should run (click-through).
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
    assert_eq!(app.models().get_copied(&observer_down), Some(1));
    assert_eq!(app.models().get_copied(&underlay_down), Some(1));

    // Start a dock drag session for a different pointer id: suppression is window-global.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

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
        app.models().get_copied(&observer_down),
        Some(1),
        "expected dock drag to suppress observer outside-press dispatch"
    );
    assert_eq!(app.models().get_copied(&underlay_down), Some(2));
}

#[test]
fn outside_press_observer_is_canceled_for_touch_drags() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let clicks = app.models_mut().insert(0u32);
    let observer = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer.clone(),
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
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(30.0), Px(10.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                ..fret_core::MouseButtons::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(30.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Touch,
        }),
    );

    assert_eq!(app.models().get_copied(&observer), Some(0));
}

#[test]
fn outside_press_observer_is_suppressed_during_dock_drag() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_clicks = app.models_mut().insert(0u32);
    let observer_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(ClickCounter {
        clicks: underlay_clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Establish baseline: without a dock drag, outside-press observer should run and still allow
    // click-through to reach the underlay.
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
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(app.models().get_copied(&underlay_clicks), Some(1));
    assert_eq!(app.models().get_copied(&observer_down), Some(1));

    // Start a dock drag session for this window (different pointer id).
    app.begin_drag_with_kind(
        fret_core::PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // With an active dock drag, outside-press observer must not run (ADR 0072), but hit-tested
    // dispatch should still reach the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(20.0), Px(20.0)),
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
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(20.0), Px(20.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&underlay_clicks), Some(2));
    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(1),
        "expected outside-press observer to be suppressed during dock drag"
    );
}

#[test]
fn outside_press_observer_is_suppressed_while_other_pointer_is_captured_in_different_layer() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
            }
        }
    }

    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }
    }

    let window = AppWindowId::default();
    let mut app = crate::test_host::TestHost::new();
    let observer_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CaptureOnDown);
    ui.set_root(base);

    let overlay_root = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_visible(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // First pointer establishes capture in the underlay.
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

    // Reset the observer counter; we only care about the second pointer while capture is active.
    let _ = app
        .models_mut()
        .update(&observer_down, |v: &mut u32| *v = 0);

    // Second pointer should not dismiss overlays while the first pointer is captured elsewhere.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(20.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(0),
        "expected outside-press observer dispatch to be suppressed while another pointer is captured"
    );
}

#[test]
fn outside_press_observer_works_with_view_cache_root_and_prepaint_reuse() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }

        fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
            let center = Point::new(Px(5.0), Px(5.0));
            let rotate = Transform2D::rotation_about_degrees(90.0, center);
            let translate = Transform2D::translation(Point::new(Px(40.0), Px(0.0)));
            Some(translate * rotate)
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let Some(&child) = cx.children.first() else {
                return cx.available;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            let _ = cx.layout_in(child, child_bounds);
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(cx.bounds.origin, Size::new(Px(10.0), Px(10.0)));
            cx.paint(child, child_bounds);
        }
    }

    struct CountNormalDown {
        normal: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for CountNormalDown {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.normal, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay_down = app.models_mut().insert(0u32);
    let overlay_down = app.models_mut().insert(0u32);
    let observer_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let base = ui.create_node(CountNormalDown {
        normal: underlay_down.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    ui.set_node_view_cache_flags(overlay_root, true, false, false);

    let overlay_leaf = ui.create_node(CountNormalDown {
        normal: overlay_down.clone(),
    });
    ui.add_child(overlay_root, overlay_leaf);

    let layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    // Frame 0: establish interaction recording for the view-cache overlay root.
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    // Frame 1: should be able to reuse the cached interaction range for the overlay root.
    app.advance_frame();
    // Force a non-stable layout pass so the test exercises interaction-cache replay.
    //
    // The layout engine can legitimately skip work on a completely stable frame, which would
    // bypass prepaint recording/replay and make `interaction_cache_hits` remain 0.
    ui.invalidate(base, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.debug_stats().interaction_cache_hits >= 1,
        "expected prepaint interaction cache to hit for clean view-cache roots"
    );

    // This window-space point maps to local (6, 5) inside the overlay leaf after rotation+translation.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(45.0), Px(6.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(0),
        "expected inside click to not trigger observer outside-press dispatch"
    );
    assert_eq!(
        app.models().get_copied(&overlay_down),
        Some(1),
        "expected overlay leaf to receive normal pointer down"
    );
    assert_eq!(
        app.models().get_copied(&underlay_down),
        Some(0),
        "expected underlay to not receive pointer down when overlay handles it"
    );

    // Clicking outside the overlay should trigger observer dispatch, but still reach the underlay (click-through).
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
        app.models().get_copied(&observer_down),
        Some(1),
        "expected outside click to trigger observer outside-press dispatch"
    );
    assert_eq!(app.models().get_copied(&underlay_down), Some(1));
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

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
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
        .insert(fret_runtime::InputDispatchPhase::Bubble);
    let normal_phase = app
        .models_mut()
        .insert(fret_runtime::InputDispatchPhase::Preview);

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
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_phase),
        Some(fret_runtime::InputDispatchPhase::Preview),
        "observer pass should tag InputContext as Observer"
    );
    assert_eq!(
        app.models().get_copied(&normal_phase),
        Some(fret_runtime::InputDispatchPhase::Bubble),
        "normal hit-tested dispatch should tag InputContext as Normal"
    );
}

#[test]
fn outside_press_observer_dispatches_only_topmost_dismissible_non_modal_overlay() {
    struct RecordOutsidePress {
        observer_down: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordOutsidePress {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer_down, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let a_down = app.models_mut().insert(0u32);
    let b_down = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(TestStack);
    ui.set_root(base);

    let overlay_a = ui.create_node(RecordOutsidePress {
        observer_down: a_down.clone(),
    });
    let layer_a = ui.push_overlay_root_ex(overlay_a, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer_a, true);

    let overlay_b = ui.create_node(RecordOutsidePress {
        observer_down: b_down.clone(),
    });
    let layer_b = ui.push_overlay_root_ex(overlay_b, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer_b, true);

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
        app.models().get_copied(&b_down),
        Some(1),
        "expected topmost overlay to receive outside-press observer dispatch"
    );
    assert_eq!(
        app.models().get_copied(&a_down),
        Some(0),
        "expected lower overlays to not receive outside-press observer dispatch"
    );
}

#[test]
fn outside_press_branches_can_exempt_triggers_outside_layer_subtree() {
    struct TriggerCounter {
        downs: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for TriggerCounter {
        fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
            position.x.0 <= 20.0 && position.y.0 <= 20.0
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.downs, |v: &mut u32| *v += 1);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    struct RecordOutsidePress {
        observer_down: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordOutsidePress {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
                return;
            }
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer_down, |v: &mut u32| *v += 1);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let trigger_downs = app.models_mut().insert(0u32);
    let submenu_observer_downs = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(TestStack);
    ui.set_root(base);

    let parent_root = ui.create_node(TestStack);
    let trigger = ui.create_node(TriggerCounter {
        downs: trigger_downs.clone(),
    });
    ui.add_child(parent_root, trigger);
    ui.push_overlay_root_ex(parent_root, false, true);

    let submenu_root = ui.create_node(RecordOutsidePress {
        observer_down: submenu_observer_downs.clone(),
    });
    let submenu_layer = ui.push_overlay_root_ex(submenu_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(submenu_layer, true);
    ui.set_layer_pointer_down_outside_branches(submenu_layer, vec![trigger]);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Clicking the trigger (outside the submenu layer subtree) must not count as an outside-press
    // for the submenu overlay when the trigger is registered as a branch.
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

    assert_eq!(app.models().get_copied(&trigger_downs), Some(1));
    assert_eq!(
        app.models().get_copied(&submenu_observer_downs),
        Some(0),
        "expected branch click to not trigger submenu outside-press observer dispatch"
    );

    // Clicking away from the trigger should count as an outside-press for the submenu overlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(90.0), Px(90.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&submenu_observer_downs), Some(1));
}

#[test]
fn outside_press_observer_can_suppress_hit_test_dispatch() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }
    }

    struct PressToClickCounter {
        pressed: bool,
        clicks: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for PressToClickCounter {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            match event {
                Event::Pointer(PointerEvent::Down {
                    button: fret_core::MouseButton::Left,
                    ..
                }) => {
                    self.pressed = true;
                }
                Event::Pointer(PointerEvent::Up {
                    button: fret_core::MouseButton::Left,
                    ..
                }) => {
                    if self.pressed {
                        self.pressed = false;
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&self.clicks, |v: &mut u32| *v += 1);
                        cx.stop_propagation();
                    }
                }
                _ => {}
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let observer_down = app.models_mut().insert(0u32);
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(PressToClickCounter {
        pressed: false,
        clicks: clicks.clone(),
    });
    ui.set_root(base);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_consume_pointer_down_outside_events(layer, true);

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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(1),
        "expected outside click to trigger observer outside-press dispatch"
    );
    assert_eq!(
        app.models().get_copied(&clicks),
        Some(0),
        "expected suppression to prevent underlay click activation"
    );
}

#[test]
fn outside_press_observer_suppression_respects_dismissable_branches() {
    struct RecordObserverDown {
        observer: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for RecordObserverDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.observer, |v: &mut u32| *v += 1);
            }
        }
    }

    struct PressToClickCounter {
        pressed: bool,
        clicks: Model<u32>,
    }

    impl<H: UiHost> Widget<H> for PressToClickCounter {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            match event {
                Event::Pointer(PointerEvent::Down {
                    button: fret_core::MouseButton::Left,
                    ..
                }) => {
                    self.pressed = true;
                }
                Event::Pointer(PointerEvent::Up {
                    button: fret_core::MouseButton::Left,
                    ..
                }) => {
                    if self.pressed {
                        self.pressed = false;
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&self.clicks, |v: &mut u32| *v += 1);
                        cx.stop_propagation();
                    }
                }
                _ => {}
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let observer_down = app.models_mut().insert(0u32);
    let clicks = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let branch = ui.create_node(PressToClickCounter {
        pressed: false,
        clicks: clicks.clone(),
    });
    ui.set_root(branch);

    let overlay = ui.create_node(RecordObserverDown {
        observer: observer_down.clone(),
    });
    let layer = ui.push_overlay_root_ex(overlay, false, true);
    ui.set_layer_wants_pointer_down_outside_events(layer, true);
    ui.set_layer_consume_pointer_down_outside_events(layer, true);
    ui.set_layer_pointer_down_outside_branches(layer, vec![branch]);

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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&observer_down),
        Some(0),
        "expected branch click to not trigger observer outside-press dispatch"
    );
    assert_eq!(
        app.models().get_copied(&clicks),
        Some(1),
        "expected branch click to reach underlay normally"
    );
}
