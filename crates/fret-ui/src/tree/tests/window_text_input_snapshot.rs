use super::*;

#[test]
fn paint_publishes_window_text_input_snapshot_for_focused_text_widget() {
    #[derive(Default)]
    struct SnapshotWidget {
        ime_cursor_area: Option<Rect>,
    }

    impl<H: UiHost> Widget<H> for SnapshotWidget {
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

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            // Simulate the widget computing the best-effort IME cursor area during paint.
            self.ime_cursor_area = Some(Rect::new(
                Point::new(Px(7.0), Px(9.0)),
                Size::new(Px(11.0), Px(13.0)),
            ));
            let _ = cx;
        }

        fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
            Some(fret_runtime::WindowTextInputSnapshot {
                focus_is_text_input: false,
                is_composing: true,
                text_len_utf16: 6,
                selection_utf16: Some((1, 3)),
                marked_utf16: Some((3, 5)),
                ime_cursor_area: self.ime_cursor_area,
            })
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(TestStack);
    let text = ui.create_node(SnapshotWidget::default());
    ui.add_child(root, text);
    ui.set_root(root);

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

    assert_eq!(ui.focus(), Some(text));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");

    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert_eq!(snapshot.text_len_utf16, 6);
    assert_eq!(snapshot.selection_utf16, Some((1, 3)));
    assert_eq!(snapshot.marked_utf16, Some((3, 5)));
    assert_eq!(
        snapshot.ime_cursor_area,
        Some(Rect::new(
            Point::new(Px(7.0), Px(9.0)),
            Size::new(Px(11.0), Px(13.0)),
        ))
    );
}

#[test]
fn snapshot_resets_when_focus_is_not_text_input() {
    #[derive(Default)]
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

        fn platform_text_input_snapshot(&self) -> Option<fret_runtime::WindowTextInputSnapshot> {
            Some(fret_runtime::WindowTextInputSnapshot {
                focus_is_text_input: true,
                is_composing: true,
                text_len_utf16: 3,
                selection_utf16: Some((0, 1)),
                marked_utf16: None,
                ime_cursor_area: None,
            })
        }
    }

    #[derive(Default)]
    struct FocusNonTextOnDown;

    impl<H: UiHost> Widget<H> for FocusNonTextOnDown {
        fn hit_test(&self, _bounds: Rect, _position: Point) -> bool {
            true
        }

        fn is_focusable(&self) -> bool {
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

    #[derive(Default)]
    struct SplitVertical;

    impl<H: UiHost> Widget<H> for SplitVertical {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let h = Px(cx.available.height.0 / 2.0);
            for (i, &child) in cx.children.iter().enumerate() {
                let y = Px((i as f32) * h.0);
                let bounds = Rect::new(
                    Point::new(cx.bounds.origin.x, Px(cx.bounds.origin.y.0 + y.0)),
                    Size::new(cx.available.width, h),
                );
                let _ = cx.layout_in(child, bounds);
            }
            cx.available
        }

        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            for &child in cx.children {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                } else {
                    cx.paint(child, cx.bounds);
                }
            }
        }
    }

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(SplitVertical::default());
    let text = ui.create_node(FocusTextOnDown::default());
    let button = ui.create_node(FocusNonTextOnDown::default());
    ui.add_child(root, text);
    ui.add_child(root, button);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Focus the text input.
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

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(snapshot.focus_is_text_input);
    assert!(snapshot.is_composing);
    assert_eq!(snapshot.selection_utf16, Some((0, 1)));

    // Focus a non-text widget and repaint; the snapshot should reset.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: Point::new(Px(10.0), Px(90.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert_eq!(ui.focus(), Some(button));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    let snapshot = app
        .global::<fret_runtime::WindowTextInputSnapshotService>()
        .and_then(|svc| svc.snapshot(window))
        .cloned()
        .expect("expected a window text input snapshot");
    assert!(!snapshot.focus_is_text_input);
    assert!(!snapshot.is_composing);
    assert_eq!(snapshot.text_len_utf16, 0);
    assert_eq!(snapshot.selection_utf16, None);
    assert_eq!(snapshot.marked_utf16, None);
    assert_eq!(snapshot.ime_cursor_area, None);
}
