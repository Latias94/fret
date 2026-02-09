use super::*;

#[test]
fn layout_engine_solve_stats_are_per_call_and_bounded_for_two_viewport_roots() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let model = app.models_mut().insert(vec![0.5, 0.5]);
    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-engine-solve-stats-bounded",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| {
                vec![
                    cx.flex(crate::element::FlexProps::default(), |cx| {
                        vec![cx.text("left")]
                    }),
                    cx.flex(crate::element::FlexProps::default(), |cx| {
                        vec![cx.text("right")]
                    }),
                ]
            })]
        },
    );
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let s1 = ui.debug_stats().layout_engine_solves;
    assert!(
        (1..=64).contains(&s1),
        "expected a small, non-zero solve count; got {s1}"
    );

    // A second call with identical inputs should not report the cumulative engine totals from the
    // prior call.
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    let s2 = ui.debug_stats().layout_engine_solves;
    assert_eq!(s2, 0, "expected per-call solve stats (not cumulative)");

    // Change the window bounds; this must force some engine work again, and should still be
    // bounded and per-call.
    let bounds2 = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(60.0)),
    );
    ui.layout_all(&mut app, &mut text, bounds2, 1.0);
    let s3 = ui.debug_stats().layout_engine_solves;
    assert!(
        (1..=64).contains(&s3),
        "expected a small, non-zero solve count after bounds change; got {s3}"
    );

    ui.layout_all(&mut app, &mut text, bounds2, 1.0);
    let s4 = ui.debug_stats().layout_engine_solves;
    assert_eq!(s4, 0, "expected per-call solve stats (not cumulative)");
}

#[test]
fn probe_layout_does_not_prune_layout_engine_nodes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(60.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "probe-layout-does-not-prune-engine-nodes",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("hello")])],
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let container_node = ui.children(root)[0];
    let text_node = ui.children(container_node)[0];
    assert!(
        ui.layout_engine_has_node(text_node),
        "expected a final layout to register nodes in the layout engine"
    );

    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );
    assert!(
        ui.layout_engine_has_node(text_node),
        "expected probe layouts to avoid pruning layout engine nodes"
    );
}

#[test]
fn solve_barrier_flow_root_reuses_solved_root_even_after_other_solves() {
    struct PrecomputesSameRootTwice {
        a: NodeId,
        b: NodeId,
        rect: Rect,
    }

    impl<H: UiHost> Widget<H> for PrecomputesSameRootTwice {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            cx.solve_barrier_child_root(self.a, self.rect);
            cx.solve_barrier_child_root(self.b, self.rect);
            cx.solve_barrier_child_root(self.a, self.rect);

            cx.available
        }
    }

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

    let a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-a",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("a"), cx.text("aa")])],
    );
    let b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-b",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("b")])],
    );

    let rect = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(150.0), Px(40.0)),
    );
    let parent = ui.create_node(PrecomputesSameRootTwice { a, b, rect });
    ui.set_children(parent, vec![a, b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        2,
        "expected the third precompute to reuse cached solve results"
    );
}

#[test]
fn solve_barrier_flow_root_if_needed_skips_translation_only_bounds_changes() {
    struct PrecomputeThenTranslate {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenTranslate {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let rect = if self.calls == 0 {
                cx.solve_barrier_child_root(self.child, self.rect_a);
                self.rect_a
            } else {
                cx.solve_barrier_child_root_if_needed(self.child, self.rect_b);
                self.rect_b
            };
            self.calls = self.calls.saturating_add(1);

            let _ = cx.layout_in(self.child, rect);
            cx.available
        }
    }

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

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-translate-child",
        |cx| vec![cx.container(Default::default(), |cx| vec![cx.text("a"), cx.text("b")])],
    );

    let rect_a = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(150.0), Px(40.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(10.0), Px(15.0)),
        Size::new(Px(150.0), Px(40.0)),
    );

    let parent = ui.create_node(PrecomputeThenTranslate {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "expected the first precompute to solve at least once"
    );

    // Force the parent to re-run layout within the same frame, while keeping the child subtree
    // clean. A translation-only bounds change for the child should not trigger an engine solve.
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "expected translation-only layout to avoid triggering engine solves"
    );

    let child_bounds = ui.debug_node_bounds(child).expect("child bounds");
    assert!((child_bounds.origin.y.0 - rect_b.origin.y.0).abs() < 0.01);
}

#[test]
fn layout_engine_v2_scales_px_styles_with_scale_factor() {
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

    fn run(scale_factor: f32) -> (Rect, Rect) {
        let mut app = TestHost::new();
        let mut ui: UiTree<TestHost> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            fret_core::Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(140.0)),
        );
        let viewport = Rect::new(
            fret_core::Point::new(Px(5.0), Px(3.0)),
            Size::new(Px(200.0), Px(100.0)),
        );

        let mut text = FakeTextService::default();

        let child_root = render_root(
            &mut ui,
            &mut app,
            &mut text,
            window,
            bounds,
            "scale-factor-px-style",
            |cx| {
                let flex = crate::element::FlexProps {
                    direction: fret_core::Axis::Vertical,
                    gap: Px(8.0),
                    padding: fret_core::Edges::all(Px(10.0)),
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let child = |cx: &mut ElementContext<'_, TestHost>| {
                    let props = crate::element::ContainerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Px(Px(10.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    cx.container(props, |_cx| vec![])
                };

                vec![cx.flex(flex, |cx| vec![child(cx), child(cx)])]
            },
        );

        let base = ui.create_node(RegistersViewportRoot { viewport });
        ui.set_children(base, vec![child_root]);
        ui.set_root(base);

        ui.layout_all(&mut app, &mut text, bounds, scale_factor);

        let flex_node = ui.children(child_root)[0];
        let first = ui.children(flex_node)[0];
        let second = ui.children(flex_node)[1];

        (
            ui.debug_node_bounds(first).expect("first bounds"),
            ui.debug_node_bounds(second).expect("second bounds"),
        )
    }

    let (first_1x, second_1x) = run(1.0);
    let (first_2x, second_2x) = run(2.0);

    assert_eq!(first_1x, first_2x, "expected scale-factor invariant bounds");
    assert_eq!(
        second_1x, second_2x,
        "expected scale-factor invariant bounds"
    );

    let expected_first = Rect::new(
        fret_core::Point::new(Px(15.0), Px(13.0)),
        Size::new(Px(180.0), Px(10.0)),
    );
    let expected_second = Rect::new(
        fret_core::Point::new(Px(15.0), Px(31.0)),
        Size::new(Px(180.0), Px(10.0)),
    );

    assert_eq!(first_1x, expected_first);
    assert_eq!(second_1x, expected_second);
}

#[test]
fn stack_does_not_stretch_spacer_children_in_engine_tree() {
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
        "stack-engine-no-stretch",
        |cx| {
            let mut props = crate::element::StackProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Fill;

            vec![cx.stack_props(props, |cx| {
                vec![
                    cx.spacer(crate::element::SpacerProps::default()),
                    cx.spacer(crate::element::SpacerProps::default()),
                ]
            })]
        },
    );

    let base = ui.create_node(RegistersViewportRoot { viewport });
    ui.set_children(base, vec![child_root]);
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let stack = ui.children(child_root)[0];
    let a = ui.children(stack)[0];
    let b = ui.children(stack)[1];

    let a_bounds = ui.debug_node_bounds(a).expect("a bounds");
    let b_bounds = ui.debug_node_bounds(b).expect("b bounds");

    assert_eq!(a_bounds.origin, viewport.origin);
    assert_eq!(b_bounds.origin, viewport.origin);

    assert!(a_bounds.size.width.0.abs() < 0.01);
    assert!(a_bounds.size.height.0.abs() < 0.01);
    assert!(b_bounds.size.width.0.abs() < 0.01);
    assert!(b_bounds.size.height.0.abs() < 0.01);
}

#[test]
fn positioned_container_precomputes_flow_islands_for_multiple_children() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![
            cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                vec![
                    cx.hover_region(
                        crate::element::HoverRegionProps::default(),
                        |cx, _hovered| vec![cx.text("left")],
                    ),
                    cx.flex(
                        crate::element::FlexProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            direction: fret_core::Axis::Horizontal,
                            ..Default::default()
                        },
                        |cx| vec![cx.text("right")],
                    ),
                ]
            }),
        ]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "positioned-container-multi-child-flow-islands",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let region = ui.children(root)[0];
    let hover = ui.children(region)[0];
    let flex = ui.children(region)[1];
    let hover_text = ui.children(hover)[0];
    let flex_text = ui.children(flex)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(hover_text).is_some());
    assert!(engine.layout_id_for_node(flex).is_some());
    assert!(engine.layout_id_for_node(flex_text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn nested_flow_is_solved_once_per_island() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    fn build_root(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let outer = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Vertical,
            ..Default::default()
        };

        let inner = crate::element::FlexProps {
            layout: crate::element::LayoutStyle {
                size: crate::element::SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: fret_core::Axis::Horizontal,
            gap: Px(4.0),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
        })]
    }

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "nested-flow-solve-count",
        build_root,
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let engine = ui.take_layout_engine();
    assert_eq!(
        engine.solve_count(),
        1,
        "expected nested flex subtree to be solved once as a single flow island"
    );
    ui.put_layout_engine(engine);
}
