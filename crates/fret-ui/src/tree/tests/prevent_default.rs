use super::*;
use crate::widget::{ScrollIntoViewCx, ScrollIntoViewResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
fn pointer_driven_focus_requests_do_not_scroll_into_view() {
    struct SpyScroll {
        scroll_calls: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for SpyScroll {
        fn can_scroll_descendant_into_view(&self) -> bool {
            true
        }

        fn scroll_descendant_into_view(
            &mut self,
            _cx: &mut ScrollIntoViewCx<'_, H>,
            _descendant_bounds: Rect,
        ) -> ScrollIntoViewResult {
            self.scroll_calls.fetch_add(1, Ordering::SeqCst);
            ScrollIntoViewResult::Handled { did_scroll: true }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let bounds = cx.bounds;
            for &child in cx.children {
                let _ = cx.layout_in(child, bounds);
            }
            cx.available
        }
    }

    struct FocusOnPointerUp;

    impl<H: UiHost> Widget<H> for FocusOnPointerUp {
        fn is_focusable(&self) -> bool {
            true
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Up { .. })) {
                cx.request_focus(cx.node);
            }
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let scroll_calls = Arc::new(AtomicUsize::new(0));

    let mut ui = UiTree::new();
    ui.set_window(window);
    let scroll = ui.create_node(SpyScroll {
        scroll_calls: scroll_calls.clone(),
    });
    let leaf = ui.create_node(FocusOnPointerUp);
    ui.set_root(scroll);
    ui.add_child(scroll, leaf);

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
            click_count: 1,
            is_click: true,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert_eq!(
        scroll_calls.load(Ordering::SeqCst),
        0,
        "expected pointer-driven focus requests to avoid scroll-into-view"
    );
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

#[test]
fn stop_propagation_does_not_suppress_default_focus_on_pointer_down() {
    struct FocusableStack;

    impl<H: UiHost> Widget<H> for FocusableStack {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn is_focusable(&self) -> bool {
            true
        }
    }

    struct StopPropagationLeaf;

    impl<H: UiHost> Widget<H> for StopPropagationLeaf {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.available
        }

        fn event(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
            if matches!(event, Event::Pointer(PointerEvent::Down { .. })) {
                cx.stop_propagation();
            }
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);

    let root = ui.create_node(FocusableStack);
    let leaf = ui.create_node(StopPropagationLeaf);
    ui.set_root(root);
    ui.set_children(root, vec![leaf]);

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
fn touch_focus_on_pointer_down_requests_virtual_keyboard_for_text_input() {
    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(100.0), Px(100.0))
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);
    let root = ui.create_node(FocusableTextInput);
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

    assert_eq!(ui.focus(), Some(root));

    let effects = app.take_effects();
    assert!(
        effects.iter().any(|e| {
            matches!(
                e,
                fret_runtime::Effect::ImeRequestVirtualKeyboard {
                    window: w,
                    visible: true
                } if *w == window
            )
        }),
        "touch focus on text input should request the virtual keyboard"
    );
}

#[test]
fn mouse_focus_on_pointer_down_does_not_request_virtual_keyboard_for_text_input() {
    struct FocusableTextInput;

    impl<H: UiHost> Widget<H> for FocusableTextInput {
        fn layout(&mut self, _cx: &mut LayoutCx<'_, H>) -> Size {
            Size::new(Px(100.0), Px(100.0))
        }

        fn is_focusable(&self) -> bool {
            true
        }

        fn is_text_input(&self) -> bool {
            true
        }
    }

    let window = AppWindowId::default();

    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let mut ui = UiTree::new();
    ui.set_window(window);
    let root = ui.create_node(FocusableTextInput);
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

    let effects = app.take_effects();
    assert!(
        !effects
            .iter()
            .any(|e| matches!(e, fret_runtime::Effect::ImeRequestVirtualKeyboard { .. })),
        "mouse focus should not request the virtual keyboard"
    );
}
