use super::*;

#[test]
fn viewport_rects_do_not_couple_fill_semantics_across_subtrees() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_viewport(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        let mut props = crate::element::FlexProps::default();
        props.layout.size.width = crate::element::Length::Fill;
        props.layout.size.height = crate::element::Length::Fill;

        vec![cx.flex(props, |cx| {
            let mut child = crate::element::ContainerProps::default();
            child.layout.size.width = crate::element::Length::Fill;
            child.layout.size.height = crate::element::Length::Fill;
            vec![cx.container(child, |_| Vec::new())]
        })]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-a",
        build_viewport,
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-b",
        build_viewport,
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let container_a = ui.children(flex_a)[0];
    let b_a = ui
        .debug_node_bounds(container_a)
        .expect("viewport a container bounds");
    assert!((b_a.origin.x.0 - viewport_a.origin.x.0).abs() < 0.01);
    assert!((b_a.size.width.0 - viewport_a.size.width.0).abs() < 0.01);

    let flex_b = ui.children(root_b)[0];
    let container_b = ui.children(flex_b)[0];
    let b_b = ui
        .debug_node_bounds(container_b)
        .expect("viewport b container bounds");
    assert!((b_b.origin.x.0 - viewport_b.origin.x.0).abs() < 0.01);
    assert!((b_b.size.width.0 - viewport_b.size.width.0).abs() < 0.01);
}

#[test]
fn viewport_root_registration_is_recorded_in_layout_all() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-a",
        |cx| vec![cx.text("a")],
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-b",
        |cx| vec![cx.text("b")],
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert_eq!(ui.viewport_roots().len(), 2);
    assert!(ui.viewport_roots().contains(&(root_a, viewport_a)));
    assert!(ui.viewport_roots().contains(&(root_b, viewport_b)));
}

#[test]
fn resizable_panel_group_does_not_register_viewport_roots_during_probe_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

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
        "panel-group-probe-does-not-register-viewport-roots",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| vec![cx.text("left"), cx.text("right")])]
        },
    );
    ui.set_root(root);
    ui.layout_all_with_pass_kind(
        &mut app,
        &mut text,
        bounds,
        1.0,
        crate::layout_pass::LayoutPassKind::Probe,
    );

    assert!(
        ui.viewport_roots().is_empty(),
        "expected probe layout to avoid registering viewport roots"
    );
}

#[test]
fn viewport_root_flush_only_lays_out_invalidated_roots() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountsLayout {
        count: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for CountsLayout {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            self.count.fetch_add(1, Ordering::Relaxed);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let left_count = Arc::new(AtomicUsize::new(0));
    let right_count = Arc::new(AtomicUsize::new(0));

    let left = ui.create_node(CountsLayout {
        count: left_count.clone(),
    });
    ui.set_children(left, Vec::new());
    let right = ui.create_node(CountsLayout {
        count: right_count.clone(),
    });
    ui.set_children(right, Vec::new());

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![left, right]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(left_count.load(Ordering::Relaxed), 1);
    assert_eq!(right_count.load(Ordering::Relaxed), 1);

    // Invalidate only the left root subtree; the parent must re-register viewport roots, but the
    // flush loop should only lay out the invalidated root.
    ui.invalidate(left, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(left_count.load(Ordering::Relaxed), 2);
    assert_eq!(
        right_count.load(Ordering::Relaxed),
        1,
        "expected right viewport root to be skipped when clean"
    );
}

#[test]
fn viewport_root_request_build_keeps_engine_nodes_alive_when_skipped() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    fn build_viewport(cx: &mut ElementContext<'_, TestHost>) -> Vec<AnyElement> {
        vec![cx.container(Default::default(), |cx| vec![cx.text("hello")])]
    }

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-keeps-engine-nodes-a",
        build_viewport,
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-keeps-engine-nodes-b",
        build_viewport,
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let b_container = ui.children(root_b)[0];
    let b_text = ui.children(b_container)[0];
    assert!(
        ui.layout_engine_has_node(b_text),
        "expected viewport subtree nodes to be registered in the engine after layout"
    );

    // Only invalidate the left viewport; the right root should be skipped by the flush loop, but
    // still kept alive via the request/build phase.
    ui.invalidate(root_a, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert!(
        ui.layout_engine_has_node(b_text),
        "expected skipped viewport roots to remain registered in the engine (stable identity)"
    );
}

#[test]
fn resizable_panel_group_viewport_roots_match_panel_bounds() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

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
        "panel-group-viewport-roots-match-panel-bounds",
        |cx| {
            let props = crate::element::ResizablePanelGroupProps::new(
                fret_core::Axis::Horizontal,
                model.clone(),
            );
            vec![cx.resizable_panel_group(props, |cx| vec![cx.text("left"), cx.text("right")])]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let group = ui.children(root)[0];
    let panel_a = ui.children(group)[0];
    let panel_b = ui.children(group)[1];

    let bounds_a = ui
        .debug_node_bounds(panel_a)
        .expect("expected panel a bounds");
    let bounds_b = ui
        .debug_node_bounds(panel_b)
        .expect("expected panel b bounds");

    assert_eq!(ui.viewport_roots().len(), 2);
    assert!(ui.viewport_roots().contains(&(panel_a, bounds_a)));
    assert!(ui.viewport_roots().contains(&(panel_b, bounds_b)));
}

#[test]
fn viewport_root_registration_is_flushed_when_registered_from_another_viewport_root() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct RegistersViewport {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for RegistersViewport {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

    struct MarksLayout {
        did_layout: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for MarksLayout {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            self.did_layout.store(true, Ordering::Relaxed);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(200.0)),
    );
    let mut text = FakeTextService::default();

    let base = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-registration-is-flushed-nested",
        |cx| vec![cx.text("base")],
    );

    let inner_did_layout = Arc::new(AtomicBool::new(false));
    let inner = ui.create_node(MarksLayout {
        did_layout: inner_did_layout.clone(),
    });
    ui.set_children(inner, Vec::new());

    let outer_viewport = Rect::new(
        Point::new(Px(10.0), Px(10.0)),
        Size::new(Px(200.0), Px(80.0)),
    );
    let inner_viewport = Rect::new(
        Point::new(Px(20.0), Px(30.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let outer = ui.create_node(RegistersViewport {
        viewport: inner_viewport,
        child: inner,
    });
    ui.set_children(outer, Vec::new());

    let root = ui.create_node(RegistersViewport {
        viewport: outer_viewport,
        child: outer,
    });
    ui.set_children(root, vec![base]);
    ui.set_root(root);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        inner_did_layout.load(Ordering::Relaxed),
        "expected nested viewport root to be laid out in the same frame"
    );
    assert!(ui.viewport_roots().contains(&(outer, outer_viewport)));
    assert!(ui.viewport_roots().contains(&(inner, inner_viewport)));
}

#[test]
fn viewport_root_layout_is_applied_before_overlay_root_layout() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
        saw_default_bounds: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            let bounds = cx
                .tree
                .debug_node_bounds(self.child)
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            self.saw_default_bounds
                .store(bounds.size == Size::default(), Ordering::Relaxed);
            cx.available
        }
    }

    struct OverlayReadsViewportBounds {
        viewport: Rect,
        child: NodeId,
        saw_expected_bounds: Arc<AtomicBool>,
    }

    impl<H: UiHost> Widget<H> for OverlayReadsViewportBounds {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let bounds = cx
                .tree
                .debug_node_bounds(self.child)
                .unwrap_or_else(|| Rect::new(Point::new(Px(0.0), Px(0.0)), Size::default()));
            let ok = (bounds.origin.x.0 - self.viewport.origin.x.0).abs() < 0.01
                && (bounds.origin.y.0 - self.viewport.origin.y.0).abs() < 0.01
                && (bounds.size.width.0 - self.viewport.size.width.0).abs() < 0.01
                && (bounds.size.height.0 - self.viewport.size.height.0).abs() < 0.01;
            self.saw_expected_bounds.store(ok, Ordering::Relaxed);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-overlay-order-child",
        |cx| vec![cx.text("child")],
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let saw_default_bounds = Arc::new(AtomicBool::new(false));
    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child,
        saw_default_bounds: saw_default_bounds.clone(),
    });
    ui.set_root(base);

    let saw_expected_bounds = Arc::new(AtomicBool::new(false));
    let overlay = ui.create_node(OverlayReadsViewportBounds {
        viewport,
        child,
        saw_expected_bounds: saw_expected_bounds.clone(),
    });
    ui.push_overlay_root(overlay, false);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    assert!(
        saw_default_bounds.load(Ordering::Relaxed),
        "expected viewport root to be laid out after base root, not during Base.layout()"
    );
    assert!(
        saw_expected_bounds.load(Ordering::Relaxed),
        "expected viewport root bounds to be available during overlay root layout"
    );
}

#[test]
fn viewport_roots_do_not_couple_fill_layout() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(400.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root_a = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-fill-a",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
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
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );
    let root_b = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-fill-b",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
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
            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps::default())]
            })]
        },
    );

    let viewport_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let viewport_b = Rect::new(
        Point::new(Px(120.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );

    let parent = ui.create_node(TwoViewportRects::new(viewport_a, viewport_b));
    ui.set_children(parent, vec![root_a, root_b]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let flex_a = ui.children(root_a)[0];
    let spacer_a = ui.children(flex_a)[0];
    let a = ui
        .debug_node_bounds(spacer_a)
        .expect("viewport a spacer bounds");
    assert!((a.size.width.0 - viewport_a.size.width.0).abs() < 0.01);
    assert!((a.size.height.0 - viewport_a.size.height.0).abs() < 0.01);

    let flex_b = ui.children(root_b)[0];
    let spacer_b = ui.children(flex_b)[0];
    let b = ui
        .debug_node_bounds(spacer_b)
        .expect("viewport b spacer bounds");
    assert!((b.size.width.0 - viewport_b.size.width.0).abs() < 0.01);
    assert!((b.size.height.0 - viewport_b.size.height.0).abs() < 0.01);
}

#[test]
fn viewport_root_nested_flow_is_solved_once() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-nested-flow-solve-count",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let engine = ui.take_layout_engine();
    assert_eq!(
        engine.solve_count(),
        1,
        "expected viewport root flow subtree to be solved exactly once"
    );
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_hover_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let inner_a = crate::element::FlexProps {
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

        let inner_b = crate::element::FlexProps {
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
            vec![cx.hover_region(
                crate::element::HoverRegionProps::default(),
                |cx, _hovered| {
                    vec![
                        cx.flex(inner_a, |cx| vec![cx.text("hello")]),
                        cx.flex(inner_b, |cx| vec![cx.text("world")]),
                    ]
                },
            )]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-hover-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let hover = ui.children(outer)[0];
    let inner_a = ui.children(hover)[0];
    let inner_b = ui.children(hover)[1];
    let text_a = ui.children(inner_a)[0];
    let text_b = ui.children(inner_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(hover).is_some());
    assert!(engine.layout_id_for_node(inner_a).is_some());
    assert!(engine.layout_id_for_node(inner_b).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_pointer_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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
            vec![
                cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let inner = ui.children(region)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_pointer_region_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let a = crate::element::FlexProps {
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
        };

        let b = crate::element::FlexProps {
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
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.pointer_region(crate::element::PointerRegionProps::default(), |cx| {
                    vec![
                        cx.flex(a, |cx| vec![cx.text("a")]),
                        cx.flex(b, |cx| vec![cx.text("b")]),
                    ]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let flex_a = ui.children(region)[0];
    let flex_b = ui.children(region)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_pointer_region_absolute_child_fills_region() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let mut region = crate::element::PointerRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        let mut barrier = crate::element::ContainerProps::default();
        barrier.layout.position = crate::element::PositionStyle::Absolute;
        barrier.layout.inset.top = Some(Px(0.0));
        barrier.layout.inset.right = Some(Px(0.0));
        barrier.layout.inset.bottom = Some(Px(0.0));
        barrier.layout.inset.left = Some(Px(0.0));
        barrier.layout.size.width = Length::Fill;
        barrier.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.pointer_region(region, |cx| vec![cx.container(barrier, |_cx| vec![])])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pointer-region-absolute-only",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let barrier = ui.children(region)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");

    assert_eq!(region_bounds, viewport);
    assert_eq!(barrier_bounds, viewport);
}

#[test]
fn viewport_root_pressable_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        vec![cx.flex(outer, |cx| {
            vec![
                cx.pressable(crate::element::PressableProps::default(), |cx, _state| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-pressable-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let pressable = ui.children(outer)[0];
    let a = ui.children(pressable)[0];
    let b = ui.children(pressable)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(pressable).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_semantics_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        vec![cx.flex(outer, |cx| {
            vec![
                cx.semantics(crate::element::SemanticsProps::default(), |cx| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-semantics-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let semantics = ui.children(outer)[0];
    let a = ui.children(semantics)[0];
    let b = ui.children(semantics)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(semantics).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_focus_scope_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        vec![cx.flex(outer, |cx| {
            vec![
                cx.focus_scope(crate::element::FocusScopeProps::default(), |cx| {
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-focus-scope-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let focus = ui.children(outer)[0];
    let a = ui.children(focus)[0];
    let b = ui.children(focus)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(focus).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_opacity_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        vec![cx.flex(outer, |cx| {
            vec![cx.opacity(0.5, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-opacity-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let opacity = ui.children(outer)[0];
    let a = ui.children(opacity)[0];
    let b = ui.children(opacity)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(opacity).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_visual_transform_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let transform =
            fret_core::Transform2D::translation(fret_core::Point::new(Px(1.0), Px(2.0)));
        vec![cx.flex(outer, |cx| {
            vec![cx.visual_transform(transform, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-visual-transform-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let transform = ui.children(outer)[0];
    let a = ui.children(transform)[0];
    let b = ui.children(transform)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(transform).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_interactivity_gate_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        vec![cx.flex(outer, |cx| {
            vec![cx.interactivity_gate(true, true, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-interactivity-gate-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let gate = ui.children(outer)[0];
    let a = ui.children(gate)[0];
    let b = ui.children(gate)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(gate).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_container_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let mut container = crate::element::ContainerProps::default();
        container.layout.size.width = Length::Fill;
        container.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.container(container, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-container-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let container = ui.children(outer)[0];
    let a = ui.children(container)[0];
    let b = ui.children(container)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(container).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_stack_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let mut stack = crate::element::StackProps::default();
        stack.layout.size.width = Length::Fill;
        stack.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.stack_props(stack, |cx| vec![cx.text("a"), cx.text("b")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-stack-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let stack = ui.children(outer)[0];
    let a = ui.children(stack)[0];
    let b = ui.children(stack)[1];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(stack).is_some());
    assert!(engine.layout_id_for_node(a).is_some());
    assert!(engine.layout_id_for_node(b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_render_transform_passthrough_fill_does_not_collapse() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let transform = Transform2D::translation(Point::new(Px(5.0), Px(0.0)));

        let mut region = crate::element::PointerRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.render_transform(transform, |cx| {
                vec![cx.pointer_region(region, |_cx| vec![])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-render-transform-passthrough-fill",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let wrapper = ui.children(outer)[0];
    let region = ui.children(wrapper)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    assert_eq!(region_bounds.origin, viewport.origin);
    assert_eq!(region_bounds.size, viewport.size);
}

#[test]
fn viewport_root_wheel_region_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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
            vec![
                cx.wheel_region(crate::element::WheelRegionProps::default(), |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let inner = ui.children(region)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_auto_wrapper_promotes_fill_when_flow_child_requests_fill() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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
            align: CrossAlign::Start,
            ..Default::default()
        };

        let wrapper = crate::element::ContainerProps::default();

        let fill_child = crate::element::FlexProps {
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
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.container(wrapper, |cx| {
                vec![cx.flex(fill_child, |cx| vec![cx.text("fill-child")])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-auto-wrapper-promotes-fill",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let wrapper = ui.children(outer)[0];

    let wrapper_bounds = ui.debug_node_bounds(wrapper).expect("wrapper bounds");
    assert_eq!(wrapper_bounds.origin, viewport.origin);
    assert_eq!(wrapper_bounds.size.width, viewport.size.width);
    assert!(wrapper_bounds.size.width.0 > 0.0);

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(wrapper).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_wheel_region_wraps_multiple_children_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let a = crate::element::FlexProps {
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
        };

        let b = crate::element::FlexProps {
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
        };

        vec![cx.flex(outer, |cx| {
            vec![
                cx.wheel_region(crate::element::WheelRegionProps::default(), |cx| {
                    vec![
                        cx.flex(a, |cx| vec![cx.text("a")]),
                        cx.flex(b, |cx| vec![cx.text("b")]),
                    ]
                }),
            ]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-multi-child-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let flex_a = ui.children(region)[0];
    let flex_b = ui.children(region)[1];
    let text_a = ui.children(flex_a)[0];
    let text_b = ui.children(flex_b)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(region).is_some());
    assert!(engine.layout_id_for_node(flex_a).is_some());
    assert!(engine.layout_id_for_node(text_a).is_some());
    assert!(engine.layout_id_for_node(flex_b).is_some());
    assert!(engine.layout_id_for_node(text_b).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_wheel_region_absolute_child_fills_region() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let mut region = crate::element::WheelRegionProps::default();
        region.layout.size.width = Length::Fill;
        region.layout.size.height = Length::Fill;

        let mut barrier = crate::element::ContainerProps::default();
        barrier.layout.position = crate::element::PositionStyle::Absolute;
        barrier.layout.inset.top = Some(Px(0.0));
        barrier.layout.inset.right = Some(Px(0.0));
        barrier.layout.inset.bottom = Some(Px(0.0));
        barrier.layout.inset.left = Some(Px(0.0));
        barrier.layout.size.width = Length::Fill;
        barrier.layout.size.height = Length::Fill;

        vec![cx.flex(outer, |cx| {
            vec![cx.wheel_region(region, |cx| vec![cx.container(barrier, |_cx| vec![])])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-wheel-region-absolute-only",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let region = ui.children(outer)[0];
    let barrier = ui.children(region)[0];

    let region_bounds = ui.debug_node_bounds(region).expect("region bounds");
    let barrier_bounds = ui.debug_node_bounds(barrier).expect("barrier bounds");

    assert_eq!(region_bounds, viewport);
    assert_eq!(barrier_bounds, viewport);
}

#[test]
fn viewport_root_roving_flex_is_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let mut roving = crate::element::RovingFlexProps::default();
        roving.flex.layout.size.width = Length::Fill;
        roving.flex.layout.size.height = Length::Auto;
        roving.flex.direction = fret_core::Axis::Horizontal;
        roving.flex.gap = Px(4.0);

        vec![cx.flex(outer, |cx| {
            vec![cx.roving_flex(roving, |cx| vec![cx.text("hello")])]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-roving-flex-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let roving = ui.children(outer)[0];
    let text = ui.children(roving)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(roving).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn viewport_root_anchored_wraps_flow_in_engine_tree() {
    struct BaseRegistersViewportRoot {
        viewport: Rect,
        child: NodeId,
    }

    impl<H: UiHost> Widget<H> for BaseRegistersViewportRoot {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let _ = cx.layout_viewport_root(self.child, self.viewport);
            cx.available
        }
    }

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

        let anchored = crate::element::AnchoredProps {
            anchor: Rect::new(
                Point::new(Px(10.0), Px(10.0)),
                Size::new(Px(10.0), Px(10.0)),
            ),
            ..Default::default()
        };

        vec![cx.flex(outer, |cx| {
            vec![cx.anchored_props(anchored, |cx| {
                vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
            })]
        })]
    }

    let viewport_child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "viewport-root-anchored-flow-tree",
        build_root,
    );

    let viewport = Rect::new(
        Point::new(Px(10.0), Px(5.0)),
        Size::new(Px(120.0), Px(40.0)),
    );

    let base = ui.create_node(BaseRegistersViewportRoot {
        viewport,
        child: viewport_child,
    });
    ui.set_root(base);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let outer = ui.children(viewport_child)[0];
    let anchored = ui.children(outer)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(engine.layout_id_for_node(anchored).is_some());
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn overlay_root_dismissible_layer_precomputes_child_flow_islands() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );

    let base = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-underlay",
        |cx| vec![cx.text("underlay")],
    );
    ui.set_root(base);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-dismissible-precompute",
        |cx| {
            let anchored = crate::element::AnchoredProps {
                anchor: Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
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

            vec![cx.anchored_props(anchored, |cx| {
                vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
            })]
        },
    );
    let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let anchored = ui.children(overlay_root)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(
        engine.layout_id_for_node(overlay_root).is_some(),
        "expected dismissible root to be represented in the engine tree"
    );
    assert!(
        engine.layout_id_for_node(anchored).is_some(),
        "expected overlay subtree to be precomputed into the engine tree"
    );
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn overlay_root_scroll_precomputes_child_flow_islands() {
    let window = AppWindowId::default();
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let mut services = FakeTextService::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );

    let base = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-scroll-underlay",
        |cx| vec![cx.text("underlay")],
    );
    ui.set_root(base);

    let overlay_root = crate::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "overlay-root-scroll-precompute",
        |cx| {
            let anchored = crate::element::AnchoredProps {
                anchor: Rect::new(
                    Point::new(Px(10.0), Px(10.0)),
                    Size::new(Px(10.0), Px(10.0)),
                ),
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

            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = Length::Fill;
            scroll.layout.size.height = Length::Fill;
            scroll.probe_unbounded = true;

            vec![cx.scroll(scroll, |cx| {
                vec![cx.anchored_props(anchored, |cx| {
                    vec![cx.flex(inner, |cx| vec![cx.text("hello")])]
                })]
            })]
        },
    );
    let _layer = ui.push_overlay_root_ex(overlay_root, false, true);

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scroll = ui.children(overlay_root)[0];
    let anchored = ui.children(scroll)[0];
    let inner = ui.children(anchored)[0];
    let text = ui.children(inner)[0];

    let engine = ui.take_layout_engine();
    assert!(
        engine.layout_id_for_node(anchored).is_some(),
        "expected scroll to precompute its child flow island into the engine tree"
    );
    assert!(engine.layout_id_for_node(inner).is_some());
    assert!(engine.layout_id_for_node(text).is_some());
    ui.put_layout_engine(engine);
}

#[test]
fn fixed_split_registers_viewport_roots_to_avoid_widget_fallback_solves() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(40.0)),
    );
    let mut text = FakeTextService::default();

    let left = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fixed-split-left",
        |cx| {
            vec![cx.flex(
                crate::element::FlexProps {
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = Length::Fill;
                        l.size.height = Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| vec![cx.text("left")],
            )]
        },
    );

    let right = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "fixed-split-right",
        |cx| {
            vec![cx.grid(
                crate::element::GridProps {
                    cols: 1,
                    layout: {
                        let mut l = crate::element::LayoutStyle::default();
                        l.size.width = Length::Fill;
                        l.size.height = Length::Fill;
                        l
                    },
                    ..Default::default()
                },
                |cx| vec![cx.text("right")],
            )]
        },
    );

    let split = crate::FixedSplit::create_node_with_children(
        &mut ui,
        crate::FixedSplit::horizontal(0.5),
        left,
        right,
    );
    ui.set_root(split);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);
    assert_eq!(ui.debug_stats().layout_engine_widget_fallback_solves, 0);
}
