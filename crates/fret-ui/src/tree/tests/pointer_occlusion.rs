use super::*;

#[derive(Clone)]
struct Counts {
    moves: Model<u32>,
    downs: Model<u32>,
    wheels: Model<u32>,
}

struct CounterWidget {
    counts: Counts,
}

impl<H: UiHost> Widget<H> for CounterWidget {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        true
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        match event {
            Event::Pointer(PointerEvent::Move { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.moves, |v: &mut u32| *v += 1);
            }
            Event::Pointer(PointerEvent::Down { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.downs, |v: &mut u32| *v += 1);
            }
            Event::Pointer(PointerEvent::Wheel { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.counts.wheels, |v: &mut u32| *v += 1);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct HitTestTransparent;

impl<H: UiHost> Widget<H> for HitTestTransparent {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct OutsidePressObserverCounter {
    observer_downs: Model<u32>,
}

impl<H: UiHost> Widget<H> for OutsidePressObserverCounter {
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
                .update(&self.observer_downs, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct PointerMoveObserverCounter {
    observer_moves: Model<u32>,
}

impl<H: UiHost> Widget<H> for PointerMoveObserverCounter {
    fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
        false
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if matches!(event, Event::Pointer(PointerEvent::Move { .. })) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.observer_moves, |v: &mut u32| *v += 1);
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

struct CornerCaptureOverlay {
    moves: Model<u32>,
    downs: Model<u32>,
    observer_downs: Model<u32>,
}

impl<H: UiHost> Widget<H> for CornerCaptureOverlay {
    fn hit_test(&self, _bounds: Rect, position: Point) -> bool {
        position.x.0 <= 20.0 && position.y.0 <= 20.0
    }

    fn event_observer(&mut self, cx: &mut crate::widget::ObserverCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Preview {
            return;
        }
        if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
            let _ = cx
                .app
                .models_mut()
                .update(&self.observer_downs, |v: &mut u32| *v += 1);
        }
    }

    fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        if cx.input_ctx.dispatch_phase != fret_runtime::InputDispatchPhase::Bubble {
            return;
        }

        match event {
            Event::Pointer(PointerEvent::Down { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.downs, |v: &mut u32| *v += 1);
                cx.capture_pointer(cx.node);
                cx.stop_propagation();
            }
            Event::Pointer(PointerEvent::Move { .. }) => {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&self.moves, |v: &mut u32| *v += 1);
            }
            _ => {}
        }
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        cx.available
    }
}

#[test]
fn pointer_occlusion_block_mouse_except_scroll_suppresses_underlay_hit_dispatch_but_allows_wheel() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    // An overlay layer that occludes underlay pointer interactions, but allows scroll.
    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let hit = ui.debug_hit_test(Point::new(Px(10.0), Px(10.0)));
    assert_eq!(hit.hit, Some(base), "expected underlay to be hit-testable");
    assert_eq!(hit.barrier_root, None, "expected no modal barrier");

    // Move and down should not reach the underlay while occlusion is active.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
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

    // Wheel should still route to the underlay scroll target when configured as "except scroll".
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 1);
}

#[test]
fn pointer_occlusion_block_mouse_suppresses_underlay_hit_dispatch_including_wheel() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouse);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 0);
}

#[test]
fn pointer_occlusion_block_mouse_except_scroll_is_window_global_across_pointers() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(HitTestTransparent);
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(10.0), Px(10.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&counts.moves).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&counts.wheels).unwrap_or(0), 2);
}

#[test]
fn pointer_occlusion_does_not_suppress_outside_press_observer_dispatch() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let observer_downs = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    // A hit-test-transparent overlay that requests outside-press observer events and enables
    // pointer occlusion (Radix `disableOutsidePointerEvents` outcome).
    let overlay_root = ui.create_node(OutsidePressObserverCounter {
        observer_downs: observer_downs.clone(),
    });
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

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

    assert_eq!(app.models().get_copied(&observer_downs).unwrap_or(0), 1);
    assert_eq!(
        app.models().get_copied(&counts.downs).unwrap_or(0),
        0,
        "expected pointer occlusion to block underlay pointer-down dispatch"
    );
}

#[test]
fn pointer_occlusion_allows_pointer_move_observer_dispatch_while_suppressing_underlay_hit_dispatch() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let counts = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let observer_moves = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: counts.clone(),
    });
    ui.set_root(base);

    let overlay_root = ui.create_node(PointerMoveObserverCounter {
        observer_moves: observer_moves.clone(),
    });
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_move_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), None);
    assert_eq!(ui.captured(), None);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(10.0), Px(10.0)),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        app.models().get_copied(&counts.moves).unwrap_or(0),
        0,
        "expected pointer occlusion to suppress underlay pointer-move dispatch"
    );
    assert_eq!(
        app.models().get_copied(&observer_moves).unwrap_or(0),
        1,
        "expected pointer occlusion to still dispatch pointer-move observer events to overlays"
    );
    assert_eq!(ui.focus(), None, "observer pass must not change focus");
    assert_eq!(
        ui.captured(),
        None,
        "observer pass must not capture pointers"
    );
}

#[test]
fn pointer_occlusion_respects_pointer_capture_for_one_pointer_but_occludes_others() {
    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    let underlay = Counts {
        moves: app.models_mut().insert(0u32),
        downs: app.models_mut().insert(0u32),
        wheels: app.models_mut().insert(0u32),
    };
    let overlay_moves = app.models_mut().insert(0u32);
    let overlay_downs = app.models_mut().insert(0u32);
    let observer_downs = app.models_mut().insert(0u32);

    let mut ui = UiTree::new();
    ui.set_window(window);

    let base = ui.create_node(CounterWidget {
        counts: underlay.clone(),
    });
    ui.set_root(base);

    // A small overlay hit region (top-left corner) captures pointer 0 on down, while the layer
    // occludes underlay pointer interaction outside the overlay.
    let overlay_root = ui.create_node(CornerCaptureOverlay {
        moves: overlay_moves.clone(),
        downs: overlay_downs.clone(),
        observer_downs: observer_downs.clone(),
    });
    let overlay_layer = ui.push_overlay_root_ex(overlay_root, false, true);
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, true);
    ui.set_layer_pointer_occlusion(overlay_layer, PointerOcclusion::BlockMouseExceptScroll);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Pointer 0 starts inside the overlay hit region and is captured by the overlay.
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
        Some(overlay_root),
        "expected overlay to capture pointer 0"
    );

    // Even after leaving the overlay hit region, pointer 0 should continue to route to the
    // captured overlay node.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Move {
            position: Point::new(Px(50.0), Px(50.0)),
            buttons: fret_core::MouseButtons {
                left: true,
                ..Default::default()
            },
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Pointer 1 is not captured. A down outside the overlay should:
    // - trigger outside-press observer dispatch to the overlay,
    // - be suppressed for the underlay due to pointer occlusion.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(50.0), Px(50.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Wheel remains allowed for non-captured pointers in BlockMouseExceptScroll mode.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Wheel {
            position: Point::new(Px(50.0), Px(50.0)),
            delta: fret_core::Point::new(Px(0.0), Px(-10.0)),
            modifiers: fret_core::Modifiers::default(),
            pointer_id: fret_core::PointerId(1),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(app.models().get_copied(&overlay_downs).unwrap_or(0), 1);
    assert_eq!(app.models().get_copied(&overlay_moves).unwrap_or(0), 1);
    assert_eq!(app.models().get_copied(&observer_downs).unwrap_or(0), 1);

    assert_eq!(app.models().get_copied(&underlay.downs).unwrap_or(0), 0);
    assert_eq!(app.models().get_copied(&underlay.wheels).unwrap_or(0), 1);
}
