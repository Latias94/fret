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
