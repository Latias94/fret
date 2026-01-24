use super::*;

#[test]
fn focus_on_pointer_down_defaults_to_first_focusable_ancestor() {
    struct Focusable;

    impl<H: UiHost> Widget<H> for Focusable {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(100.0), Px(100.0))
        }

        fn is_focusable(&self) -> bool {
            true
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);
    let root = ui.create_node(Focusable);
    ui.set_root(root);

    let mut services = FakeUiServices;
    ui.layout_all(
        &mut app,
        &mut services,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        1.0,
    );

    assert_eq!(ui.focus(), None);

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

    assert_eq!(ui.focus(), Some(root));
}

#[test]
fn prevent_default_focus_on_pointer_down_suppresses_default_focus() {
    struct PreventFocus;

    impl<H: UiHost> Widget<H> for PreventFocus {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(100.0), Px(100.0))
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.prevent_default(fret_runtime::DefaultAction::FocusOnPointerDown);
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);
    let root = ui.create_node(PreventFocus);
    ui.set_root(root);

    let mut services = FakeUiServices;
    ui.layout_all(
        &mut app,
        &mut services,
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        ),
        1.0,
    );

    assert_eq!(ui.focus(), None);

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

    assert_eq!(ui.focus(), None);
}
