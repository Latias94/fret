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

    let arbitration = app
        .global::<fret_runtime::WindowInputArbitrationService>()
        .and_then(|svc| svc.snapshot(window))
        .copied()
        .expect("expected a window input arbitration snapshot");
    assert!(arbitration.pointer_capture_active);
    assert_eq!(arbitration.pointer_capture_root, Some(layer_root));
    assert!(!arbitration.pointer_capture_multiple_roots);
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

    let arbitration = app
        .global::<fret_runtime::WindowInputArbitrationService>()
        .and_then(|svc| svc.snapshot(window))
        .copied()
        .expect("expected a window input arbitration snapshot");
    assert_eq!(
        arbitration.modal_barrier_root,
        Some(overlay_root),
        "expected modal barrier root to be reflected in the arbitration snapshot",
    );
}
