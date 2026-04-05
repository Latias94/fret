use super::*;

#[test]
fn dispatch_event_publishes_post_dispatch_input_arbitration_snapshot() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
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
    let capture = ui.create_node(CaptureOnDown);
    ui.add_child(root, capture);
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

    assert_eq!(ui.captured_for(fret_core::PointerId(0)), Some(capture));
    let layer = ui
        .node_layer(capture)
        .expect("expected capture node to be attached to a layer");
    let layer_root = ui.layer_root(layer).expect("expected layer root");

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert!(arbitration.pointer_capture_active);
    assert_eq!(arbitration.pointer_capture_root, Some(layer_root));
    assert!(!arbitration.pointer_capture_multiple_roots);
}

#[test]
fn escape_cancels_dock_drag_and_publishes_post_dispatch_input_arbitration_snapshot() {
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

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    app.begin_cross_window_drag_with_kind(
        fret_core::PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
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
            pointer_id: fret_core::PointerId(7),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.captured_for(fret_core::PointerId(7)), Some(anchor));

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
        ui.captured_for(fret_core::PointerId(7)),
        None,
        "expected dock-drag capture to be cleared when the session is canceled"
    );

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert!(
        !arbitration.pointer_capture_active,
        "dock-drag cancel should publish an arbitration snapshot without stale pointer capture"
    );
    assert_eq!(arbitration.pointer_capture_root, None);
}

#[test]
fn dispatch_command_publishes_post_dispatch_input_arbitration_snapshot() {
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

    ui.dispatch_command(&mut app, &mut services, &CommandId::from("test.open_modal"));

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert_eq!(
        arbitration.modal_barrier_root,
        Some(overlay_root),
        "expected modal barrier root to be reflected in the arbitration snapshot",
    );
}

#[test]
fn imperative_focus_barrier_mutation_requires_explicit_window_snapshot_commit() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    ui.set_root(base_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.publish_window_runtime_snapshots(&mut app);

    let initial = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected an initial window input context snapshot");
    let initial_arbitration = initial
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert_eq!(initial_arbitration.focus_barrier_root, None);

    ui.set_layer_blocks_underlay_focus(overlay_layer, true);

    let stale = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a stale window input context snapshot");
    let stale_arbitration = stale
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert_eq!(
        stale_arbitration.focus_barrier_root, None,
        "raw focus-barrier mutation should not silently republish arbitration snapshots"
    );

    ui.publish_window_runtime_snapshots(&mut app);

    let committed = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a committed window input context snapshot");
    let committed_arbitration = committed
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert_eq!(
        committed_arbitration.focus_barrier_root,
        Some(overlay_root),
        "explicit window snapshot commit should publish the authoritative focus barrier root"
    );
}

#[test]
fn layout_all_focus_barrier_mutation_publishes_post_layout_input_arbitration_snapshot() {
    struct EnableBarrierDuringLayout {
        overlay_layer: UiLayerId,
    }

    impl<H: UiHost> Widget<H> for EnableBarrierDuringLayout {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.tree
                .set_layer_blocks_underlay_focus(self.overlay_layer, true);
            cx.available
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let overlay_root = ui.create_node(TestStack);
    let overlay_layer = ui.push_overlay_root_with_options(
        overlay_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: false,
            hit_testable: false,
        },
    );
    let base_root = ui.create_node(EnableBarrierDuringLayout { overlay_layer });
    ui.set_root(base_root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot after layout-time barrier mutation");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");
    assert_eq!(
        arbitration.focus_barrier_root,
        Some(overlay_root),
        "layout-time focus-barrier changes should publish the authoritative arbitration snapshot at the final layout boundary"
    );
}

#[test]
fn modal_barrier_hides_pointer_occlusion_layers_below_barrier_in_arbitration_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    let base_layer = ui.set_base_root(base_root);
    ui.set_layer_pointer_occlusion(base_layer, PointerOcclusion::BlockMouse);

    let modal_root = ui.create_node(TestStack);
    ui.push_overlay_root_with_options(
        modal_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: true,
            hit_testable: false,
        },
    );

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

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");

    assert_eq!(arbitration.modal_barrier_root, Some(modal_root));
    assert_eq!(
        arbitration.pointer_occlusion,
        fret_runtime::WindowPointerOcclusion::None,
        "expected pointer occlusion layers below the barrier to be ignored",
    );
    assert_eq!(
        arbitration.pointer_occlusion_root, None,
        "expected no occlusion root when only the underlay is occluding",
    );
}

#[test]
fn pointer_occlusion_above_modal_barrier_is_reported_in_arbitration_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let base_root = ui.create_node(TestStack);
    ui.set_base_root(base_root);

    let modal_root = ui.create_node(TestStack);
    ui.push_overlay_root_with_options(
        modal_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: true,
            hit_testable: false,
        },
    );

    let top_root = ui.create_node(TestStack);
    let top_layer = ui.push_overlay_root(top_root, false);
    ui.set_layer_pointer_occlusion(top_layer, PointerOcclusion::BlockMouseExceptScroll);

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

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");

    assert_eq!(arbitration.modal_barrier_root, Some(modal_root));
    assert_eq!(
        arbitration.pointer_occlusion,
        fret_runtime::WindowPointerOcclusion::BlockMouseExceptScroll
    );
    assert_eq!(arbitration.pointer_occlusion_root, Some(top_root));
}

#[test]
fn modal_barrier_scopes_pointer_capture_to_active_roots() {
    struct CaptureOnDown;

    impl<H: UiHost> Widget<H> for CaptureOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.capture_pointer(cx.node);
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

    let base_root = ui.create_node(TestStack);
    let capture = ui.create_node(CaptureOnDown);
    ui.add_child(base_root, capture);
    ui.set_root(base_root);

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
    assert_eq!(ui.captured_for(fret_core::PointerId(0)), Some(capture));

    let modal_root = ui.create_node(TestStack);
    ui.push_overlay_root_with_options(
        modal_root,
        crate::OverlayRootOptions {
            blocks_underlay_input: true,
            hit_testable: false,
        },
    );

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

    let input_ctx = app
        .global::<fret_runtime::WindowInputContextService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window input context snapshot");
    let arbitration = input_ctx
        .window_arbitration
        .expect("expected `InputContext.window_arbitration` to be populated");

    assert_eq!(arbitration.modal_barrier_root, Some(modal_root));
    assert_eq!(ui.captured_for(fret_core::PointerId(0)), None);
    assert!(
        !arbitration.pointer_capture_active,
        "expected modal barrier scope enforcement to clear underlay captures"
    );
}
