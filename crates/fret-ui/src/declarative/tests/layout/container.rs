use super::*;

#[test]
fn container_does_not_stretch_spacer_child_in_engine_tree() {
    struct RegistersViewportRoot {
        viewport: Rect,
    }

    impl<H: UiHost> Widget<H> for RegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let child = cx.children[0];
            let _ = cx.layout_viewport_root(child, self.viewport);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(140.0)),
    );
    let viewport = Rect::new(
        fret_core::Point::new(Px(7.0), Px(11.0)),
        Size::new(Px(200.0), Px(100.0)),
    );

    let mut text = FakeTextService::default();

    let child_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "container-engine-no-stretch",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.container(props, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container = ui.children(child_root)[0];
    let spacer = ui.children(container)[0];

    let container_bounds = ui.debug_node_bounds(container).expect("container bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer).expect("spacer bounds");

    assert_eq!(container_bounds, viewport);
    assert_eq!(spacer_bounds.origin, viewport.origin);
    assert!(spacer_bounds.size.width.0.abs() < 0.01);
    assert!(spacer_bounds.size.height.0.abs() < 0.01);
}

#[test]
fn container_absolute_inset_positions_child() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp58-stack-absolute",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(0.0));
                    badge.layout.inset.right = Some(Px(0.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert_eq!(container_bounds.size.width, Px(100.0));
    assert_eq!(container_bounds.size.height, Px(80.0));

    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(90.0));
    assert_eq!(badge_bounds.origin.y, Px(0.0));
}

#[test]
fn container_absolute_negative_inset_offsets_outside_parent() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp62-stack-absolute-negative-inset",
        |cx| {
            vec![
                cx.container(crate::element::ContainerProps::default(), |cx| {
                    let mut base = crate::element::ContainerProps::default();
                    base.layout.size.width = crate::element::Length::Px(Px(100.0));
                    base.layout.size.height = crate::element::Length::Px(Px(80.0));

                    let mut badge = crate::element::ContainerProps::default();
                    badge.layout.size.width = crate::element::Length::Px(Px(10.0));
                    badge.layout.size.height = crate::element::Length::Px(Px(10.0));
                    badge.layout.position = crate::element::PositionStyle::Absolute;
                    badge.layout.inset.top = Some(Px(-5.0));
                    badge.layout.inset.left = Some(Px(-6.0));

                    vec![
                        cx.container(base, |_cx| vec![]),
                        cx.container(badge, |_cx| vec![]),
                    ]
                }),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let children = ui.children(container_node);
    assert_eq!(children.len(), 2);
    let badge_bounds = ui.debug_node_bounds(children[1]).expect("badge bounds");
    assert_eq!(badge_bounds.origin.x, Px(-6.0));
    assert_eq!(badge_bounds.origin.y, Px(-5.0));
}

#[test]
fn container_applies_padding_and_paints_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-container",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    padding: fret_core::Edges::symmetric(Px(4.0), Px(6.0)),
                    background: Some(Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");
    assert_eq!(text_bounds.origin.x, Px(4.0));
    assert_eq!(text_bounds.origin.y, Px(6.0));
    assert_eq!(text_bounds.size.width, Px(10.0));
    assert_eq!(text_bounds.size.height, Px(10.0));

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 2);
    match scene.ops()[0] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected quad op"),
    }
}

#[test]
fn container_border_change_invalidates_child_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let mut root: Option<NodeId> = None;
    for pass in 0..2 {
        let border = if pass == 0 { Px(0.0) } else { Px(4.0) };
        let rendered = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "container-border-change-invalidates-child-layout",
            |cx| {
                let container = crate::element::ContainerProps {
                    border: fret_core::Edges::all(border),
                    ..Default::default()
                };

                vec![cx.container(container, |cx| vec![cx.text("hi")])]
            },
        );
        let root_node = *root.get_or_insert(rendered);
        if pass == 0 {
            ui.set_root(root_node);
        } else {
            assert_eq!(
                root_node, rendered,
                "expected stable root node across frames"
            );
        }

        ui.layout_all(&mut app, &mut text, bounds, 1.0);
        let container_node = ui.children(root_node)[0];
        let text_node = ui.children(container_node)[0];
        let text_bounds = ui.debug_node_bounds(text_node).expect("text bounds");

        assert_eq!(text_bounds.origin.x, border);
        assert_eq!(text_bounds.origin.y, border);

        app.advance_frame();
    }
}

#[test]
fn container_shrink_wraps_to_max_child_under_definite_parent_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp50-container-shrink-wraps",
        |cx| {
            let outer = crate::element::ContainerProps {
                padding: fret_core::Edges::all(Px(2.0)),
                ..crate::element::ContainerProps::default()
            };

            vec![cx.container(outer, |cx| {
                let mut fixed = crate::element::ContainerProps::default();
                fixed.layout.size.width = crate::element::Length::Px(Px(30.0));
                fixed.layout.size.height = crate::element::Length::Px(Px(15.0));

                vec![cx.container(fixed, |_| Vec::new()), cx.text("x")]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(root)[0];
    let outer_bounds = ui.debug_node_bounds(outer).expect("outer bounds");
    assert_eq!(outer_bounds.size.width, Px(34.0));
    assert_eq!(outer_bounds.size.height, Px(19.0));
}

#[test]
fn container_nested_chains_do_not_trigger_extra_engine_solves_when_clean() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "container-nested-clean-solves",
        |cx| {
            let outer = crate::element::ContainerProps {
                padding: fret_core::Edges::all(Px(2.0)),
                ..Default::default()
            };

            let inner = crate::element::ContainerProps {
                padding: fret_core::Edges::all(Px(1.0)),
                ..Default::default()
            };

            vec![cx.container(outer, |cx| {
                vec![cx.container(inner, |cx| vec![cx.text("x")])]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_solves, 1);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_solves, 0);
}

#[test]
fn container_paints_shadow_before_background() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "mvp60-shadow",
        |cx| {
            vec![cx.container(
                crate::element::ContainerProps {
                    background: Some(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    shadow: Some(crate::element::ShadowStyle {
                        primary: crate::element::ShadowLayerStyle {
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.5,
                            },
                            offset_x: Px(2.0),
                            offset_y: Px(3.0),
                            blur: Px(0.0),
                            spread: Px(1.0),
                        },
                        secondary: None,
                        corner_radii: fret_core::Corners::all(Px(4.0)),
                    }),
                    corner_radii: fret_core::Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |cx| vec![cx.text("hi")],
            )]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let container_node = ui.children(root)[0];
    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut text, bounds, &mut scene, 1.0);

    assert_eq!(scene.ops_len(), 3);

    let shadow_bounds = match scene.ops()[0] {
        SceneOp::Quad { rect, .. } => rect,
        _ => panic!("expected shadow quad first"),
    };
    match scene.ops()[1] {
        SceneOp::Quad {
            rect, background, ..
        } => {
            assert_eq!(rect, container_bounds);
            assert_eq!(background.a, 1.0);
        }
        _ => panic!("expected background quad second"),
    }

    assert!(shadow_bounds.origin.x.0 > container_bounds.origin.x.0);
    assert!(shadow_bounds.origin.y.0 > container_bounds.origin.y.0);
    assert!(shadow_bounds.size.width.0 > container_bounds.size.width.0);
    assert!(shadow_bounds.size.height.0 > container_bounds.size.height.0);
}
