#![allow(clippy::arc_with_non_send_sync)]

use super::*;

#[test]
fn focus_scope_traps_focus_traversal_within_subtree() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut before: Option<GlobalElementId> = None;
    let mut scope_id: Option<GlobalElementId> = None;
    let mut inside_a: Option<GlobalElementId> = None;
    let mut inside_b: Option<GlobalElementId> = None;
    let mut after: Option<GlobalElementId> = None;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            let pressable_layout = {
                let mut layout = crate::element::LayoutStyle::default();
                layout.size.width = crate::element::Length::Px(Px(10.0));
                layout.size.height = crate::element::Length::Px(Px(10.0));
                layout
            };

            vec![cx.flex(
                crate::element::FlexProps {
                    layout: crate::element::LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: crate::element::MainAlign::Start,
                    align: crate::element::CrossAlign::Start,
                    wrap: false,
                },
                |cx| {
                    vec![
                        cx.keyed(1, |cx| {
                            cx.pressable_with_id(
                                crate::element::PressableProps {
                                    layout: pressable_layout,
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st, id| {
                                    before = Some(id);
                                    Vec::new()
                                },
                            )
                        }),
                        cx.keyed(2, |cx| {
                            cx.focus_scope_with_id(
                                crate::element::FocusScopeProps {
                                    trap_focus: true,
                                    ..Default::default()
                                },
                                |cx, id| {
                                    scope_id = Some(id);
                                    vec![
                                        cx.keyed(3, |cx| {
                                            cx.pressable_with_id(
                                                crate::element::PressableProps {
                                                    layout: pressable_layout,
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |_cx, _st, id| {
                                                    inside_a = Some(id);
                                                    Vec::new()
                                                },
                                            )
                                        }),
                                        cx.keyed(4, |cx| {
                                            cx.pressable_with_id(
                                                crate::element::PressableProps {
                                                    layout: pressable_layout,
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |_cx, _st, id| {
                                                    inside_b = Some(id);
                                                    Vec::new()
                                                },
                                            )
                                        }),
                                    ]
                                },
                            )
                        }),
                        cx.keyed(5, |cx| {
                            cx.pressable_with_id(
                                crate::element::PressableProps {
                                    layout: pressable_layout,
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st, id| {
                                    after = Some(id);
                                    Vec::new()
                                },
                            )
                        }),
                    ]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before = before.expect("before id");
    let scope_id = scope_id.expect("scope id");
    let inside_a = inside_a.expect("inside a id");
    let inside_b = inside_b.expect("inside b id");
    let after = after.expect("after id");

    let scope_node = crate::elements::node_for_element(&mut app, window, scope_id).unwrap();
    let inside_a_node = crate::elements::node_for_element(&mut app, window, inside_a).unwrap();
    let inside_b_node = crate::elements::node_for_element(&mut app, window, inside_b).unwrap();
    let before_node = crate::elements::node_for_element(&mut app, window, before).unwrap();
    let after_node = crate::elements::node_for_element(&mut app, window, after).unwrap();

    ui.set_focus(Some(inside_a_node));

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(
        ui.focus(),
        Some(inside_b_node),
        "expected focus scope to trap within subtree; scope={scope_node:?} before={before_node:?} a={inside_a_node:?} b={inside_b_node:?} after={after_node:?}"
    );

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(inside_a_node));

    let did_handle =
        ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.previous"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(inside_b_node));

    assert_ne!(ui.focus(), Some(before_node));
    assert_ne!(ui.focus(), Some(after_node));
}

#[test]
fn focus_scope_prevents_pointer_focus_from_leaving_subtree_when_trapped() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let after_clicked = app.models_mut().insert(false);

    let mut before: Option<GlobalElementId> = None;
    let mut scope_id: Option<GlobalElementId> = None;
    let mut inside_a: Option<GlobalElementId> = None;
    let mut inside_b: Option<GlobalElementId> = None;
    let mut after: Option<GlobalElementId> = None;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            let pressable_layout = {
                let mut layout = crate::element::LayoutStyle::default();
                layout.size.width = crate::element::Length::Px(Px(10.0));
                layout.size.height = crate::element::Length::Px(Px(10.0));
                layout
            };

            vec![cx.flex(
                crate::element::FlexProps {
                    layout: crate::element::LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: crate::element::MainAlign::Start,
                    align: crate::element::CrossAlign::Start,
                    wrap: false,
                },
                |cx| {
                    vec![
                        cx.keyed(1, |cx| {
                            cx.pressable_with_id(
                                crate::element::PressableProps {
                                    layout: pressable_layout,
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st, id| {
                                    before = Some(id);
                                    Vec::new()
                                },
                            )
                        }),
                        cx.keyed(2, |cx| {
                            cx.focus_scope_with_id(
                                crate::element::FocusScopeProps {
                                    trap_focus: true,
                                    ..Default::default()
                                },
                                |cx, id| {
                                    scope_id = Some(id);
                                    vec![
                                        cx.keyed(3, |cx| {
                                            cx.pressable_with_id(
                                                crate::element::PressableProps {
                                                    layout: pressable_layout,
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |_cx, _st, id| {
                                                    inside_a = Some(id);
                                                    Vec::new()
                                                },
                                            )
                                        }),
                                        cx.keyed(4, |cx| {
                                            cx.pressable_with_id(
                                                crate::element::PressableProps {
                                                    layout: pressable_layout,
                                                    enabled: true,
                                                    focusable: true,
                                                    ..Default::default()
                                                },
                                                |_cx, _st, id| {
                                                    inside_b = Some(id);
                                                    Vec::new()
                                                },
                                            )
                                        }),
                                    ]
                                },
                            )
                        }),
                        cx.keyed(5, |cx| {
                            let after_clicked = after_clicked.clone();
                            cx.pressable_with_id(
                                crate::element::PressableProps {
                                    layout: pressable_layout,
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |cx, _st, id| {
                                    after = Some(id);
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, _cx, _reason| {
                                            let _ = host
                                                .models_mut()
                                                .update(&after_clicked, |v| *v = true);
                                        },
                                    ));
                                    Vec::new()
                                },
                            )
                        }),
                    ]
                },
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scope_id = scope_id.expect("scope id");
    let inside_a = inside_a.expect("inside a id");
    let inside_b = inside_b.expect("inside b id");
    let after = after.expect("after id");

    let scope_node = crate::elements::node_for_element(&mut app, window, scope_id).unwrap();
    let inside_a_node = crate::elements::node_for_element(&mut app, window, inside_a).unwrap();
    let inside_b_node = crate::elements::node_for_element(&mut app, window, inside_b).unwrap();
    let after_node = crate::elements::node_for_element(&mut app, window, after).unwrap();

    ui.set_focus(Some(inside_a_node));

    let after_bounds = ui.debug_node_bounds(after_node).expect("after bounds");
    let click = Point::new(
        Px(after_bounds.origin.x.0 + after_bounds.size.width.0 / 2.0),
        Px(after_bounds.origin.y.0 + after_bounds.size.height.0 / 2.0),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Down {
            position: click,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let focus_after_down = ui.focus();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(PointerEvent::Up {
            position: click,
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: true,
            click_count: 1,
            pointer_id: fret_core::PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    let focus_after_up = ui.focus();

    assert_eq!(
        app.models().get_copied(&after_clicked),
        Some(true),
        "expected pointer activation to still run"
    );
    assert_ne!(focus_after_down, Some(after_node));
    assert!(
        matches!(focus_after_up, Some(n) if n == inside_a_node || n == inside_b_node),
        "expected trapped focus scope to prevent pointer focus from leaving subtree; scope={scope_node:?} after={after_node:?} focus_after_down={focus_after_down:?} focus_after_up={focus_after_up:?}"
    );
}
