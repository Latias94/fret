use super::*;

#[test]
fn dispatch_event_publishes_post_dispatch_input_context_snapshot() {
    struct FocusTextOnDown;

    impl<H: UiHost> Widget<H> for FocusTextOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.request_focus(cx.node);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(FocusTextOnDown);
    ui.add_child(root, text);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

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

    assert_eq!(ui.focus(), Some(text));

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    assert!(input_ctx.focus_is_text_input);
}

#[test]
fn timer_dispatch_publishes_post_dispatch_input_context_snapshot() {
    struct FocusTextOnTimer {
        target: NodeId,
    }

    impl<H: UiHost> Widget<H> for FocusTextOnTimer {
        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Timer { .. }) {
                cx.request_focus(self.target);
                cx.stop_propagation();
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }
    }

    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let text = ui.create_node(FocusableTextInput);
    let root = ui.create_node(FocusTextOnTimer { target: text });
    ui.add_child(root, text);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Timer {
            token: fret_core::TimerToken(7),
        },
    );

    assert_eq!(ui.focus(), Some(text));
    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    assert!(
        input_ctx.focus_is_text_input,
        "timer-driven focus changes should publish the new authoritative input context"
    );
}

#[test]
fn focused_node_text_boundary_mode_override_is_published_in_input_context_snapshot() {
    struct FocusTextOnDown;

    impl<H: UiHost> Widget<H> for FocusTextOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.request_focus(cx.node);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(FocusTextOnDown);
    ui.add_child(root, text);
    ui.set_root(root);

    ui.set_node_text_boundary_mode_override(text, Some(fret_runtime::TextBoundaryMode::Identifier));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

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

    assert_eq!(ui.focus(), Some(text));

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    assert_eq!(
        input_ctx.text_boundary_mode,
        fret_runtime::TextBoundaryMode::Identifier
    );
}

#[test]
fn imperative_focus_mutation_requires_explicit_window_snapshot_commit() {
    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(FocusableTextInput);
    ui.add_child(root, text);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);
    ui.publish_window_runtime_snapshots(&mut app);

    let initial = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected an initial window input context snapshot");
    assert!(
        !initial.focus_is_text_input,
        "baseline snapshot should not report text-input focus before the raw focus mutation"
    );

    ui.set_focus(Some(text));

    let stale = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a stale window input context snapshot");
    assert!(
        !stale.focus_is_text_input,
        "raw focus mutation should not silently republish the authoritative input context"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a committed window input context snapshot");
    assert!(
        committed.focus_is_text_input,
        "explicit window snapshot commit should publish the authoritative focus state"
    );
}

#[test]
fn outside_press_consume_publishes_post_dispatch_input_context_snapshot() {
    struct UnderlayTarget;

    impl<H: UiHost> Widget<H> for UnderlayTarget {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    struct TransparentOverlayRoot;

    impl<H: UiHost> Widget<H> for TransparentOverlayRoot {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            false
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let overlay_bounds = Rect::new(cx.bounds.origin, Size::new(Px(20.0), Px(20.0)));
                let _ = cx.layout_in(child, overlay_bounds);
            }
            Size::new(Px(20.0), Px(20.0))
        }
    }

    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let underlay = ui.create_node(UnderlayTarget);
    ui.set_root(underlay);

    let overlay_text = ui.create_node(FocusableTextInput);
    let overlay_root = ui.create_node(TransparentOverlayRoot);
    ui.add_child(overlay_root, overlay_text);
    let overlay_layer = ui.push_overlay_root(overlay_root, false);
    ui.set_layer_wants_pointer_down_outside_events(overlay_layer, true);
    ui.set_layer_consume_pointer_down_outside_events(overlay_layer, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.set_focus(Some(overlay_text));
    ui.publish_window_runtime_snapshots(&mut app);

    let before = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot before outside press");
    assert!(before.focus_is_text_input);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(80.0), Px(80.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(ui.focus(), None);
    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot after outside press");
    assert!(
        !input_ctx.focus_is_text_input,
        "outside-press focus clearing should publish the new authoritative input context"
    );
}

#[test]
fn dispatch_command_publishes_post_dispatch_input_context_snapshot() {
    struct OpenModal {
        overlay_root: NodeId,
    }

    impl<H: UiHost> Widget<H> for OpenModal {
        fn is_focusable(&self) -> bool {
            true
        }

        fn command(&mut self, cx: &mut CommandCx<'_, H>, command: &CommandId) -> bool {
            if command.as_str() != "test.open_modal" {
                return false;
            }

            cx.tree.push_overlay_root(self.overlay_root, true);
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let overlay_root = ui.create_node(TestStack);
    let opener = ui.create_node(OpenModal { overlay_root });
    ui.add_child(root, opener);
    ui.set_root(root);
    ui.set_focus(Some(opener));

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

    assert_eq!(ui.focus(), Some(opener));

    ui.dispatch_command(&mut app, &mut services, &CommandId::from("test.open_modal"));

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    assert!(
        input_ctx.ui_has_modal,
        "expected modal overlay to be reflected in the window input context snapshot"
    );
    assert_eq!(ui.focus(), None);
}

#[test]
fn paint_all_publishes_programmatic_input_context_snapshot() {
    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(FocusableTextInput);
    ui.add_child(root, text);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_in(&mut app, &mut services, root, bounds, 1.0);
    ui.set_focus(Some(text));

    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    assert!(
        input_ctx.focus_is_text_input,
        "paint should refresh the input context snapshot after programmatic focus changes"
    );
}
