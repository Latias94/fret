use super::*;
use crate::element::{ContainerProps, PressableProps};
use fret_core::{Event, KeyCode, Modifiers};
use std::sync::Arc;

#[test]
fn key_capture_hooks_propagate_to_ancestors_and_can_stop_bubble() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let capture_ran = app.models_mut().insert(false);
    let bubble_ran = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let capture_ran_for_hook = capture_ran.clone();
    let bubble_ran_for_hook = bubble_ran.clone();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "key-capture-hooks-propagate",
        move |cx| {
            let child = cx.pressable_with_id_props(move |cx, _state, id| {
                let bubble_ran = bubble_ran_for_hook.clone();
                cx.key_add_on_key_down_for(
                    id,
                    Arc::new(move |host, action_cx, down| {
                        if down.key != KeyCode::ArrowRight {
                            return false;
                        }
                        let _ = host.models_mut().update(&bubble_ran, |v| *v = true);
                        host.request_redraw(action_cx.window);
                        true
                    }),
                );
                (
                    PressableProps {
                        focusable: true,
                        ..Default::default()
                    },
                    vec![cx.text("child")],
                )
            });

            let container = cx.container(ContainerProps::default(), move |_cx| vec![child]);

            cx.key_add_on_key_down_capture_for(
                container.id,
                Arc::new(move |host, action_cx, down| {
                    if down.key != KeyCode::ArrowRight {
                        return false;
                    }
                    let _ = host
                        .models_mut()
                        .update(&capture_ran_for_hook, |v| *v = true);
                    host.request_redraw(action_cx.window);
                    true
                }),
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let child_node = ui.children(container_node)[0];
    ui.set_focus(Some(child_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.models().get_copied(&capture_ran).unwrap_or(false),
        "expected capture key hook to run on ancestor"
    );
    assert!(
        !app.models().get_copied(&bubble_ran).unwrap_or(false),
        "expected capture hook to stop propagation before bubble key hooks"
    );
}

#[test]
fn key_bubble_hooks_propagate_to_ancestors_when_not_handled() {
    let mut app = TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let root_ran = app.models_mut().insert(false);

    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(240.0), Px(60.0)));
    let mut services = FakeTextService::default();

    let root_ran_for_hook = root_ran.clone();
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "key-bubble-hooks-propagate",
        move |cx| {
            let child = cx.pressable_with_id_props(move |_cx, _state, _id| {
                (
                    PressableProps {
                        focusable: true,
                        ..Default::default()
                    },
                    Vec::<AnyElement>::new(),
                )
            });

            let container = cx.container(ContainerProps::default(), move |_cx| vec![child]);

            cx.key_add_on_key_down_for(
                container.id,
                Arc::new(move |host, action_cx, down| {
                    if down.key != KeyCode::ArrowRight {
                        return false;
                    }
                    let _ = host.models_mut().update(&root_ran_for_hook, |v| *v = true);
                    host.request_redraw(action_cx.window);
                    true
                }),
            );

            vec![container]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let child_node = ui.children(container_node)[0];
    ui.set_focus(Some(child_node));

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::KeyDown {
            key: KeyCode::ArrowRight,
            modifiers: Modifiers::default(),
            repeat: false,
        },
    );

    assert!(
        app.models().get_copied(&root_ran).unwrap_or(false),
        "expected bubble key hook to run on ancestor"
    );
}
