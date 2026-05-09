#![allow(clippy::arc_with_non_send_sync)]

use super::*;

#[test]
fn interactivity_gate_can_make_subtree_inert_without_unmounting() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let mut before: Option<GlobalElementId> = None;
    let mut inside: Option<GlobalElementId> = None;
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

            vec![cx.pointer_region(
                crate::element::PointerRegionProps {
                    enabled: false,
                    ..Default::default()
                },
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
                                    cx.interactivity_gate(true, false, |cx| {
                                        vec![cx.pressable_with_id(
                                            crate::element::PressableProps {
                                                layout: pressable_layout,
                                                enabled: true,
                                                focusable: true,
                                                ..Default::default()
                                            },
                                            |_cx, _st, id| {
                                                inside = Some(id);
                                                Vec::new()
                                            },
                                        )]
                                    })
                                }),
                                cx.keyed(3, |cx| {
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
            )]
        },
    );

    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before = before.expect("before id");
    let inside = inside.expect("inside id");
    let after = after.expect("after id");

    let before_node = crate::elements::node_for_element(&mut app, window, before).unwrap();
    let inside_node = crate::elements::node_for_element(&mut app, window, inside).unwrap();
    let after_node = crate::elements::node_for_element(&mut app, window, after).unwrap();

    // Focus traversal should skip the inert subtree.
    ui.set_focus(Some(before_node));
    let did_handle = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(after_node));

    let did_handle =
        ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.previous"));
    assert!(did_handle);
    assert_eq!(ui.focus(), Some(before_node));

    // Hit-testing inside the inert slot should not target its focusable child.
    let hit = ui.debug_hit_test(Point::new(Px(15.0), Px(5.0))).hit;
    assert_ne!(hit, Some(inside_node));
}

#[test]
fn absent_interactivity_gate_suppresses_hidden_layout_dirty_for_resize_reuse() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(PlatformCapabilities::default());

    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let mut services = FakeUiServices;
    let initial_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let resized_bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(200.0)),
    );
    let present = std::rc::Rc::new(std::cell::Cell::new(false));

    fn render(
        ui: &mut UiTree<crate::test_host::TestHost>,
        app: &mut crate::test_host::TestHost,
        services: &mut FakeUiServices,
        window: AppWindowId,
        bounds: Rect,
        present: std::rc::Rc<std::cell::Cell<bool>>,
    ) -> NodeId {
        declarative::render_root(ui, app, services, window, bounds, "root", move |cx| {
            let mut layout = crate::element::LayoutStyle::default();
            layout.size.width = crate::element::Length::Px(Px(10.0));
            layout.size.height = crate::element::Length::Px(Px(10.0));

            vec![cx.keyed("gate", |cx| {
                cx.interactivity_gate(present.get(), true, move |cx| {
                    vec![cx.keyed("hidden_child", |cx| {
                        cx.pressable(
                            crate::element::PressableProps {
                                layout,
                                enabled: true,
                                focusable: true,
                                ..Default::default()
                            },
                            |_cx, _st| Vec::new(),
                        )
                    })]
                })
            })]
        })
    }

    let root = render(
        &mut ui,
        &mut app,
        &mut services,
        window,
        initial_bounds,
        present.clone(),
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, initial_bounds, 1.0);

    let gate = ui.nodes[root].children[0];
    let hidden_child = ui.nodes[gate].children[0];
    assert!(
        ui.nodes[hidden_child].invalidation.layout,
        "display-none child should remain mounted and layout-dirty while absent"
    );
    assert_eq!(
        ui.nodes[gate].subtree_layout_dirty_count, 0,
        "absent gate should not expose hidden child dirty work to ancestors"
    );
    assert_eq!(
        ui.nodes[root].subtree_layout_dirty_count, 0,
        "hidden child dirty work should not keep the window root subtree-dirty"
    );

    app.advance_frame();
    let root = render(
        &mut ui,
        &mut app,
        &mut services,
        window,
        resized_bounds,
        present.clone(),
    );
    ui.layout_all(&mut app, &mut services, resized_bounds, 1.0);

    let resize_record = ui
        .debug_layout_request_build_roots()
        .iter()
        .find(|record| record.root == root)
        .expect("window root request-build record");
    assert_eq!(
        resize_record.mode, "cached_flow_reuse",
        "interactive resize should reuse cached flow when only an absent subtree is dirty"
    );
    assert_eq!(resize_record.subtree_layout_dirty_count, 0);

    present.set(true);
    app.advance_frame();
    let root = render(
        &mut ui,
        &mut app,
        &mut services,
        window,
        resized_bounds,
        present,
    );
    ui.layout_all(&mut app, &mut services, resized_bounds, 1.0);

    let gate = ui.nodes[root].children[0];
    let visible_child = ui.nodes[gate].children[0];
    assert_eq!(
        visible_child, hidden_child,
        "present transition should keep the mounted child identity"
    );
    assert!(
        !ui.nodes[visible_child].invalidation.layout,
        "present transition should consume the previously hidden child layout work"
    );
}
