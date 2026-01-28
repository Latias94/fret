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
