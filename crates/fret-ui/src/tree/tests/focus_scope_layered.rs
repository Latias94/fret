#![allow(clippy::arc_with_non_send_sync)]

use super::*;

#[test]
fn focus_scope_traps_focus_traversal_inside_modal_overlay_layer_root() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let pressable_layout = {
        let mut layout = crate::element::LayoutStyle::default();
        layout.size.width = crate::element::Length::Px(Px(10.0));
        layout.size.height = crate::element::Length::Px(Px(10.0));
        layout
    };

    let mut before: Option<GlobalElementId> = None;
    let base_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base_root",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    layout: crate::element::LayoutStyle::default(),
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: crate::element::MainAlign::Start,
                    align: crate::element::CrossAlign::Start,
                    wrap: false,
                },
                |cx| {
                    vec![cx.pressable_with_id(
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
                    )]
                },
            )]
        },
    );

    let mut scope_id: Option<GlobalElementId> = None;
    let mut inside_a: Option<GlobalElementId> = None;
    let mut inside_b: Option<GlobalElementId> = None;
    let overlay_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay_root",
        |cx| {
            vec![cx.focus_scope_with_id(
                crate::element::FocusScopeProps {
                    trap_focus: true,
                    ..Default::default()
                },
                |cx, id| {
                    scope_id = Some(id);
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
                                    inside_a = Some(id);
                                    Vec::new()
                                },
                            )
                        }),
                        cx.keyed(2, |cx| {
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
            )]
        },
    );

    ui.set_root(base_root);
    let _overlay_layer = ui.push_overlay_root(overlay_root, /* blocks_underlay_input */ true);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before = before.expect("before id");
    let scope_id = scope_id.expect("scope id");
    let inside_a = inside_a.expect("inside a id");
    let inside_b = inside_b.expect("inside b id");

    let before_node = crate::elements::node_for_element(&mut app, window, before).unwrap();
    let scope_node = crate::elements::node_for_element(&mut app, window, scope_id).unwrap();
    let inside_a_node = crate::elements::node_for_element(&mut app, window, inside_a).unwrap();
    let inside_b_node = crate::elements::node_for_element(&mut app, window, inside_b).unwrap();

    ui.set_focus(Some(inside_a_node));

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(
        ui.focus(),
        Some(inside_b_node),
        "expected focus scope to trap within overlay layer root; scope={scope_node:?} a={inside_a_node:?} b={inside_b_node:?}"
    );

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(inside_a_node));

    ui.set_focus(Some(before_node));
    assert_eq!(
        ui.focus(),
        Some(inside_a_node),
        "expected modal overlay focus barrier to reject underlay focus; underlay={before_node:?} scope={scope_node:?}"
    );
}

#[test]
fn stacked_trapped_focus_scopes_prefer_innermost_scope_across_layer_root() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );

    let pressable_layout = {
        let mut layout = crate::element::LayoutStyle::default();
        layout.size.width = crate::element::Length::Px(Px(10.0));
        layout.size.height = crate::element::Length::Px(Px(10.0));
        layout
    };

    let base_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base_root_inner_wins",
        |_cx| Vec::new(),
    );
    ui.set_root(base_root);

    let mut outer_scope_id: Option<GlobalElementId> = None;
    let mut inner_scope_id: Option<GlobalElementId> = None;
    let mut inner_a: Option<GlobalElementId> = None;
    let mut inner_b: Option<GlobalElementId> = None;
    let mut outer_sibling: Option<GlobalElementId> = None;
    let overlay_root = declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay_root_inner_wins",
        |cx| {
            vec![cx.focus_scope_with_id(
                crate::element::FocusScopeProps {
                    trap_focus: true,
                    ..Default::default()
                },
                |cx, id| {
                    outer_scope_id = Some(id);
                    vec![
                        cx.focus_scope_with_id(
                            crate::element::FocusScopeProps {
                                trap_focus: true,
                                ..Default::default()
                            },
                            |cx, id| {
                                inner_scope_id = Some(id);
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
                                                inner_a = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                    cx.keyed(2, |cx| {
                                        cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: pressable_layout,
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                inner_b = Some(id);
                                                Vec::new()
                                            },
                                        )
                                    }),
                                ]
                            },
                        ),
                        cx.keyed(3, |cx| {
                            cx.pressable_with_id(
                                crate::element::PressableProps {
                                    layout: pressable_layout,
                                    enabled: true,
                                    focusable: true,
                                    ..Default::default()
                                },
                                |_cx, _st, id| {
                                    outer_sibling = Some(id);
                                    Vec::new()
                                },
                            )
                        }),
                    ]
                },
            )]
        },
    );
    let _overlay_layer = ui.push_overlay_root(overlay_root, /* blocks_underlay_input */ true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let outer_scope_id = outer_scope_id.expect("outer scope id");
    let inner_scope_id = inner_scope_id.expect("inner scope id");
    let inner_a = inner_a.expect("inner a id");
    let inner_b = inner_b.expect("inner b id");
    let outer_sibling = outer_sibling.expect("outer sibling id");

    let outer_scope_node =
        crate::elements::node_for_element(&mut app, window, outer_scope_id).unwrap();
    let inner_scope_node =
        crate::elements::node_for_element(&mut app, window, inner_scope_id).unwrap();
    let inner_a_node = crate::elements::node_for_element(&mut app, window, inner_a).unwrap();
    let inner_b_node = crate::elements::node_for_element(&mut app, window, inner_b).unwrap();
    let outer_sibling_node =
        crate::elements::node_for_element(&mut app, window, outer_sibling).unwrap();

    ui.set_focus(Some(inner_a_node));

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(
        ui.focus(),
        Some(inner_b_node),
        "expected inner trapped scope to win; outer={outer_scope_node:?} inner={inner_scope_node:?} a={inner_a_node:?} b={inner_b_node:?} sibling={outer_sibling_node:?}"
    );

    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(
        ui.focus(),
        Some(inner_a_node),
        "expected traversal to remain within inner scope; outer={outer_scope_node:?} inner={inner_scope_node:?} a={inner_a_node:?} b={inner_b_node:?} sibling={outer_sibling_node:?}"
    );
    assert_ne!(ui.focus(), Some(outer_sibling_node));
}
