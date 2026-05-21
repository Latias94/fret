use super::*;

#[test]
fn layout_sidecar_exposes_attached_test_id_in_node_debug_and_filtering() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(80.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-sidecar-test-id-bridge",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                vec![cx.text("hello overlay").test_id("layout-sidecar-node")]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!(
        "fret-ui-layout-sidecar-test-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out_dir);

    let path = ui
        .debug_write_layout_sidecar_taffy_v1_json(
            &mut app,
            window,
            root,
            bounds,
            1.0,
            Some("layout-sidecar-node"),
            &out_dir,
            1234,
        )
        .expect("layout sidecar should be written");

    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("sidecar should be readable"))
            .expect("sidecar json should parse");

    let root_node = sidecar["taffy"]["meta"]["root"]
        .as_str()
        .expect("taffy root should be a string");
    let nodes = sidecar["taffy"]["nodes"]
        .as_array()
        .expect("taffy nodes should be an array");
    let matched = nodes
        .iter()
        .find(|node| node["debug"]["test_id"].as_str() == Some("layout-sidecar-node"))
        .expect("expected layout sidecar node with attached test_id");

    assert_eq!(
        matched["node"].as_str(),
        Some(root_node),
        "root_label_filter should be able to target an attached test_id"
    );
    assert_eq!(
        matched["debug"]["instance_kind"].as_str(),
        Some("Text"),
        "structured debug metadata should expose the instance kind"
    );
    assert!(
        matched["label"]
            .as_str()
            .is_some_and(|label| label.contains("layout-sidecar-node")),
        "human-readable labels should embed the attached test_id for grep-friendly triage"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn layout_sidecar_bounds_are_logical_px_and_not_scaled_by_scale_factor() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-sidecar-logical-px-contract",
        |cx| {
            let mut props = crate::element::ContainerProps::default();
            props.layout.size.width = Length::Px(Px(224.0));
            props.layout.size.height = Length::Px(Px(36.0));
            vec![
                cx.container(props, |_cx| Vec::new())
                    .test_id("layout-sidecar-logical-target"),
            ]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.5);

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!(
        "fret-ui-layout-sidecar-logical-px-test-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out_dir);

    let path = ui
        .debug_write_layout_sidecar_taffy_v1_json(
            &mut app,
            window,
            root,
            bounds,
            1.5,
            Some("layout-sidecar-logical-target"),
            &out_dir,
            2468,
        )
        .expect("layout sidecar should be written");

    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("sidecar should be readable"))
            .expect("sidecar json should parse");

    assert_eq!(
        sidecar["meta"]["coordinate_units"].as_str(),
        Some("logical_px"),
        "layout sidecars must declare logical pixel units"
    );
    assert_eq!(
        sidecar["meta"]["scale_factor"].as_f64(),
        Some(1.5),
        "scale_factor should remain metadata rather than pre-scaling sidecar rects"
    );
    assert_eq!(
        sidecar["meta"]["root_bounds"]["w"].as_f64(),
        Some(300.0),
        "root_bounds should remain logical px even when scale_factor is fractional"
    );

    let matched = sidecar["taffy"]["nodes"]
        .as_array()
        .expect("taffy nodes should be an array")
        .iter()
        .find(|node| node["debug"]["test_id"].as_str() == Some("layout-sidecar-logical-target"))
        .expect("expected filtered sidecar dump to expose the target node");

    assert_eq!(
        matched["abs_rect"]["w"].as_f64(),
        Some(224.0),
        "abs_rect width should match logical layout width, not 224 * 1.5 or 224 / 1.5"
    );
    assert_eq!(
        matched["abs_rect"]["h"].as_f64(),
        Some(36.0),
        "abs_rect height should match logical layout height, not 36 * 1.5 or 36 / 1.5"
    );
    assert_eq!(
        matched["local_rect"]["w"].as_f64(),
        Some(224.0),
        "local_rect should use the same logical px contract as abs_rect"
    );
    assert_eq!(
        matched["local_rect"]["h"].as_f64(),
        Some(36.0),
        "local_rect should use the same logical px contract as abs_rect"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn layout_sidecar_includes_visible_overlay_roots_and_filtering_can_target_overlay_nodes() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let base_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-sidecar-underlay-root",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                vec![cx.text("underlay").test_id("layout-sidecar-underlay-node")]
            })]
        },
    );
    ui.set_root(base_root);

    let overlay_root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-sidecar-overlay-root",
        |cx| {
            vec![cx.container(Default::default(), |cx| {
                vec![cx.text("overlay").test_id("layout-sidecar-overlay-node")]
            })]
        },
    );
    let _overlay_layer = ui.push_overlay_root(overlay_root, true);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!(
        "fret-ui-layout-sidecar-overlay-test-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out_dir);

    let path = ui
        .debug_write_layout_sidecar_taffy_v1_json(
            &mut app,
            window,
            base_root,
            bounds,
            1.0,
            Some("layout-sidecar-overlay-node"),
            &out_dir,
            5678,
        )
        .expect("layout sidecar should be written");

    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("sidecar should be readable"))
            .expect("sidecar json should parse");

    assert_eq!(
        sidecar["meta"]["captured_root_count"].as_u64(),
        Some(2),
        "sidecar metadata should report all visible layer roots"
    );

    let overlay_dump_root = sidecar["taffy"]["meta"]["root"]
        .as_str()
        .expect("taffy root should be a string");
    let overlay_nodes = sidecar["taffy"]["nodes"]
        .as_array()
        .expect("taffy nodes should be an array");
    let overlay_match = overlay_nodes
        .iter()
        .find(|node| node["debug"]["test_id"].as_str() == Some("layout-sidecar-overlay-node"))
        .expect("expected filtered sidecar dump to expose the overlay node");

    assert_eq!(
        overlay_match["node"].as_str(),
        Some(overlay_dump_root),
        "root_label_filter should search across overlay roots and narrow the dump to the matched overlay node"
    );

    let root_dumps = sidecar["taffy"]["roots"]
        .as_array()
        .expect("taffy roots should be present");
    assert_eq!(
        root_dumps.len(),
        2,
        "sidecar should include all visible layer root dumps"
    );

    let base_root_label = format!("{base_root:?}");
    let overlay_root_label = format!("{overlay_root:?}");

    let base_dump = root_dumps
        .iter()
        .find(|entry| entry["root"].as_str() == Some(base_root_label.as_str()))
        .expect("expected a dump for the base layer root");
    assert!(
        base_dump["dump"]["nodes"]
            .as_array()
            .is_some_and(|nodes| nodes.iter().any(|node| {
                node["debug"]["test_id"].as_str() == Some("layout-sidecar-underlay-node")
            })),
        "base root dump should still expose underlay test ids"
    );

    let overlay_dump = root_dumps
        .iter()
        .find(|entry| entry["root"].as_str() == Some(overlay_root_label.as_str()))
        .expect("expected a dump for the overlay layer root");
    assert_eq!(
        overlay_dump["blocks_underlay_input"].as_bool(),
        Some(true),
        "overlay root metadata should retain layer barrier information"
    );
    assert!(
        overlay_dump["dump"]["nodes"]
            .as_array()
            .is_some_and(|nodes| nodes.iter().any(|node| {
                node["debug"]["test_id"].as_str() == Some("layout-sidecar-overlay-node")
            })),
        "overlay root dump should expose overlay test ids even without selecting it as the base root"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn layout_sidecar_captures_independent_layout_roots_for_scroll_content_test_ids() {
    let mut app = TestHost::new();
    let mut ui: UiTree<TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);

    let bounds = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );
    let mut text = FakeTextService::default();

    let root = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "layout-sidecar-independent-root",
        |cx| {
            let mut scroll = crate::element::ScrollProps::default();
            scroll.layout.size.width = crate::element::Length::Fill;
            scroll.layout.size.height = crate::element::Length::Fill;
            scroll.probe_unbounded = true;

            let mut rows = crate::element::FlexProps::default();
            rows.layout.size.width = crate::element::Length::Fill;
            rows.direction = fret_core::Axis::Vertical;

            vec![cx.scroll(scroll, |cx| {
                vec![cx.flex(rows, |cx| {
                    vec![
                        cx.text("independent row")
                            .test_id("layout-sidecar-independent-row"),
                        cx.text("second row"),
                    ]
                })]
            })]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!(
        "fret-ui-layout-sidecar-independent-test-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&out_dir);

    let path = ui
        .debug_write_layout_sidecar_taffy_v1_json(
            &mut app,
            window,
            root,
            bounds,
            1.0,
            Some("layout-sidecar-independent-row"),
            &out_dir,
            91011,
        )
        .expect("layout sidecar should be written");

    let sidecar: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("sidecar should be readable"))
            .expect("sidecar json should parse");

    let matched_root = sidecar["taffy"]["meta"]["root"]
        .as_str()
        .expect("taffy root should be a string");
    let matched_nodes = sidecar["taffy"]["nodes"]
        .as_array()
        .expect("taffy nodes should be an array");
    let matched = matched_nodes
        .iter()
        .find(|node| node["debug"]["test_id"].as_str() == Some("layout-sidecar-independent-row"))
        .expect("expected filtered sidecar dump to expose the scroll content test id");
    assert_eq!(
        matched["node"].as_str(),
        Some(matched_root),
        "root_label_filter should be able to narrow the dump to a node inside an independent layout root"
    );

    let captured_root_count = sidecar["meta"]["captured_root_count"]
        .as_u64()
        .expect("captured_root_count should be present");
    let visible_layer_root_count = sidecar["meta"]["visible_layer_root_count"]
        .as_u64()
        .expect("visible_layer_root_count should be present");
    assert!(
        captured_root_count > visible_layer_root_count,
        "expected sidecar to capture extra independent layout roots beyond the visible layer roots"
    );

    let independent_root = sidecar["taffy"]["roots"]
        .as_array()
        .expect("taffy roots should be present")
        .iter()
        .find(|entry| {
            entry["kind"].as_str() == Some("independent")
                && entry["dump"]["nodes"].as_array().is_some_and(|nodes| {
                    nodes.iter().any(|node| {
                        node["debug"]["test_id"].as_str() == Some("layout-sidecar-independent-row")
                    })
                })
        })
        .expect("expected an independent layout root dump for the scroll content row");
    assert_eq!(
        independent_root["blocks_underlay_input"].as_bool(),
        Some(false),
        "independent layout roots should not masquerade as modal barriers"
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

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
    let translated_id = Arc::new(std::sync::Mutex::new(None));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "precompute-translate-child",
        |cx| {
            let translated_id = translated_id.clone();
            vec![
                cx.pressable_with_id(Default::default(), move |cx, _state, id| {
                    *translated_id.lock().unwrap() = Some(id);
                    vec![cx.text("a"), cx.text("b")]
                }),
            ]
        },
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
    let translated_id = translated_id
        .lock()
        .unwrap()
        .expect("translated element id should be recorded");
    let translated_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, translated_id)
            .expect("translated element bounds");
    assert!((translated_bounds.origin.y.0 - rect_b.origin.y.0).abs() < 0.01);
}

#[test]
fn clean_engine_solved_size_delta_propagates_geometry_without_relayouting_structure() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
        Size::new(Px(320.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();
    let first_row_id = Arc::new(std::sync::Mutex::new(None));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds,
        "clean-engine-geometry-propagation-child",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                gap: Px(1.0).into(),
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
            let first_row_id = first_row_id.clone();
            vec![cx.flex(flex, |cx| {
                (0..16)
                    .map(|idx| {
                        let first_row_id = first_row_id.clone();
                        let props = crate::element::PressableProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(Px(8.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        cx.pressable_with_id(props, move |_cx, _state, id| {
                            if idx == 0 {
                                *first_row_id.lock().unwrap() = Some(id);
                            }
                            Vec::<AnyElement>::new()
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds, 1.0);

    let performed = ui.debug_stats().layout_nodes_performed;
    assert!(
        performed <= 20,
        "clean size-delta propagation should avoid re-running structural layout; performed={performed}"
    );

    let flex_node = ui.children(child)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert!((flex_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);

    let first_row_id = first_row_id
        .lock()
        .unwrap()
        .expect("first row id should be recorded");
    let first_row_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, first_row_id)
            .expect("first row element bounds");
    assert!(
        (first_row_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "fast-path propagation must refresh current element bounds"
    );
}

#[test]
fn clean_geometry_small_resize_skips_barrier_root_engine_solve() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();
    let first_row_id = Arc::new(std::sync::Mutex::new(None));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-small-resize-child",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                gap: Px(1.0).into(),
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
            let first_row_id = first_row_id.clone();
            vec![cx.flex(flex, |cx| {
                (0..8)
                    .map(|idx| {
                        let first_row_id = first_row_id.clone();
                        let props = crate::element::PressableProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(Px(8.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        };
                        cx.pressable_with_id(props, move |_cx, _state, id| {
                            if idx == 0 {
                                *first_row_id.lock().unwrap() = Some(id);
                            }
                            Vec::<AnyElement>::new()
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "expected the initial barrier layout to solve"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "clean width-only resize should propagate geometry without a Taffy root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted clean geometry skips should not leave rejection noise in frame stats"
    );

    let flex_node = ui.children(child)[0];
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert!((flex_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);

    let first_row_id = first_row_id
        .lock()
        .unwrap()
        .expect("first row id should be recorded");
    let first_row_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, first_row_id)
            .expect("first row element bounds");
    assert!(
        (first_row_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "geometry propagation must refresh descendant element bounds after skipping solve"
    );
}

#[test]
fn clean_geometry_small_resize_propagates_through_semantics_wrapper() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();
    let first_row_id = Arc::new(std::sync::Mutex::new(None));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-semantics-wrapper-child",
        |cx| {
            let semantics = crate::element::SemanticsProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                role: fret_core::SemanticsRole::Region,
                test_id: Some(Arc::<str>::from("clean-geometry-semantics-wrapper")),
                ..Default::default()
            };
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                gap: Px(1.0).into(),
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
            let first_row_id = first_row_id.clone();
            vec![cx.semantics(semantics, |cx| {
                vec![cx.flex(flex, |cx| {
                    (0..8)
                        .map(|idx| {
                            let mut props = crate::element::ContainerProps::default();
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Px(Px(8.0));
                            let row = cx.container(props, |_cx| Vec::<AnyElement>::new());
                            if idx == 0 {
                                *first_row_id.lock().unwrap() = Some(row.id);
                            }
                            row
                        })
                        .collect::<Vec<_>>()
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "expected the initial barrier layout to solve"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "Semantics is a pure wrapper and should not force a root solve during clean width-only resize"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted clean geometry skips should not leave rejection noise in frame stats"
    );

    let performed = ui.debug_stats().layout_nodes_performed;
    assert!(
        performed <= 2,
        "Semantics clean-geometry propagation should avoid re-running wrapper/subtree layout; performed={performed}"
    );

    let semantics_node = ui.children(child)[0];
    let flex_node = ui.children(semantics_node)[0];
    let semantics_bounds = ui
        .debug_node_bounds(semantics_node)
        .expect("semantics bounds");
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert!((semantics_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);
    assert!((flex_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);

    let first_row_id = first_row_id
        .lock()
        .unwrap()
        .expect("first row id should be recorded");
    let first_row_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, first_row_id)
            .expect("first row element bounds");
    assert!(
        (first_row_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "fast-path propagation must refresh descendant element bounds through Semantics"
    );
}

#[test]
fn clean_geometry_small_resize_propagates_through_pressable_wrapper() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();
    let first_row_id = Arc::new(std::sync::Mutex::new(None));
    let pressable_id = Arc::new(std::sync::Mutex::new(None));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-pressable-wrapper-child",
        |cx| {
            let pressable = crate::element::PressableProps {
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
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                gap: Px(1.0).into(),
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
            let first_row_id = first_row_id.clone();
            let pressable_id = pressable_id.clone();
            vec![cx.pressable_with_id(pressable, move |cx, _state, id| {
                *pressable_id.lock().unwrap() = Some(id);
                vec![cx.flex(flex, |cx| {
                    (0..8)
                        .map(|idx| {
                            let mut props = crate::element::ContainerProps::default();
                            props.layout.size.width = Length::Fill;
                            props.layout.size.height = Length::Px(Px(8.0));
                            let row = cx.container(props, |_cx| Vec::<AnyElement>::new());
                            if idx == 0 {
                                *first_row_id.lock().unwrap() = Some(row.id);
                            }
                            row
                        })
                        .collect::<Vec<_>>()
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "expected the initial barrier layout to solve"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "Pressable is already modeled as a pure wrapper and should not force a root solve during clean width-only resize"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted clean geometry skips should not leave rejection noise in frame stats"
    );

    let performed = ui.debug_stats().layout_nodes_performed;
    assert!(
        performed <= 1,
        "Pressable clean-geometry propagation should avoid re-running wrapper/subtree layout; performed={performed}"
    );

    let pressable_node = ui.children(child)[0];
    let flex_node = ui.children(pressable_node)[0];
    let pressable_bounds = ui
        .debug_node_bounds(pressable_node)
        .expect("pressable bounds");
    let flex_bounds = ui.debug_node_bounds(flex_node).expect("flex bounds");
    assert!((pressable_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);
    assert!((flex_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);

    let pressable_id = pressable_id
        .lock()
        .unwrap()
        .expect("pressable id should be recorded");
    let pressable_element_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, pressable_id)
            .expect("pressable element bounds");
    assert!(
        (pressable_element_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "fast-path propagation must refresh Pressable element bounds"
    );

    let first_row_id = first_row_id
        .lock()
        .unwrap()
        .expect("first row id should be recorded");
    let first_row_bounds =
        crate::elements::current_bounds_for_element(&mut app, window, first_row_id)
            .expect("first row element bounds");
    assert!(
        (first_row_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "fast-path propagation must refresh descendant element bounds through Pressable"
    );
}

#[test]
fn clean_geometry_small_resize_runs_text_input_layout_as_side_effect_boundary() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();
    let model = app.models_mut().insert(String::from("search"));

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-text-input-boundary-child",
        |cx| {
            let stack = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };
            let model = model.clone();
            vec![cx.stack_props(stack, move |cx| {
                let mut input = TextInputProps::new(model.clone());
                input.layout.size.width = Length::Fill;
                input.layout.size.height = Length::Fill;
                vec![cx.text_input(input)]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    let prepare_calls_after_initial_layout = text.prepare_calls;
    assert!(
        prepare_calls_after_initial_layout > 0,
        "initial text input layout should measure text"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "TextInput should be a side-effect boundary that lets ancestors skip the Taffy solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted clean geometry skips should not leave rejection noise in frame stats"
    );
    assert!(
        text.prepare_calls > prepare_calls_after_initial_layout,
        "TextInput layout must still run so model/font/IME side effects stay authoritative"
    );

    let stack_node = ui.children(child)[0];
    let input_node = ui.children(stack_node)[0];
    let input_bounds = ui.debug_node_bounds(input_node).expect("input bounds");
    assert!((input_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_px_absolute_stack_overlay_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-absolute-overlay-child",
        |cx| {
            let stack = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };
            vec![cx.stack_props(stack, |cx| {
                let mut viewport = crate::element::ContainerProps::default();
                viewport.layout.size.width = Length::Fill;
                viewport.layout.size.height = Length::Fill;

                let mut gate_layout = crate::element::LayoutStyle {
                    position: crate::element::PositionStyle::Absolute,
                    inset: crate::element::InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: Some(Px(0.0)).into(),
                        bottom: Some(Px(0.0)).into(),
                        left: None.into(),
                    },
                    size: crate::element::SizeStyle {
                        width: Length::Px(Px(8.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                gate_layout.size.min_width = Some(Length::Px(Px(0.0)));
                gate_layout.size.min_height = Some(Length::Px(Px(0.0)));

                let mut scrollbar_layout = crate::element::LayoutStyle::default();
                scrollbar_layout.size.width = Length::Fill;
                scrollbar_layout.size.height = Length::Fill;
                let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                    layout: scrollbar_layout,
                    axis: crate::element::ScrollbarAxis::Vertical,
                    scroll_target: None,
                    scroll_handle: crate::scroll::ScrollHandle::default(),
                    style: crate::element::ScrollbarStyle::default(),
                });

                vec![
                    cx.container(viewport, |_cx| Vec::<AnyElement>::new()),
                    cx.interactivity_gate_props(
                        crate::element::InteractivityGateProps {
                            layout: gate_layout,
                            present: true,
                            interactive: true,
                        },
                        move |cx| vec![cx.opacity(1.0, |_cx| vec![scrollbar])],
                    ),
                ]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "px-inset absolute overlay chrome should not force the Stack root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted clean geometry skips should not leave rejection noise in frame stats"
    );

    let stack_node = ui.children(child)[0];
    let gate_node = ui.children(stack_node)[1];
    let gate_bounds = ui.debug_node_bounds(gate_node).expect("gate bounds");
    assert!((gate_bounds.origin.x.0 - (rect_b.size.width.0 - 8.0)).abs() < 0.01);
    assert!((gate_bounds.size.width.0 - 8.0).abs() < 0.01);
    assert!((gate_bounds.size.height.0 - rect_b.size.height.0).abs() < 0.01);

    let opacity_node = ui.children(gate_node)[0];
    let scrollbar_node = ui.children(opacity_node)[0];
    let scrollbar_bounds = ui
        .debug_node_bounds(scrollbar_node)
        .expect("scrollbar bounds");
    assert!((scrollbar_bounds.origin.x.0 - gate_bounds.origin.x.0).abs() < 0.01);
    assert!((scrollbar_bounds.size.width.0 - gate_bounds.size.width.0).abs() < 0.01);
    assert!((scrollbar_bounds.size.height.0 - gate_bounds.size.height.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_absent_zero_absolute_overlay_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            let rect = if self.calls == 0 {
                cx.solve_barrier_child_root(self.child, self.rect_a);
                self.rect_a
            } else {
                cx.solve_barrier_child_root_if_needed(self.child, self.rect_b);
                self.rect_b
            };
            self.calls += 1;
            let _ = cx.layout_in(self.child, rect);
            cx.available
        }
    }

    let mut app = TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut text = FakeTextService::default();
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(216.0), Px(180.0)),
    );

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-absent-zero-absolute-overlay",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| {
                    let mut viewport = crate::element::ContainerProps::default();
                    viewport.layout.size.width = Length::Fill;
                    viewport.layout.size.height = Length::Fill;

                    let gate_layout = crate::element::LayoutStyle {
                        position: crate::element::PositionStyle::Absolute,
                        inset: crate::element::InsetStyle {
                            top: Some(Px(0.0)).into(),
                            right: Some(Px(0.0)).into(),
                            ..Default::default()
                        },
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(0.0)),
                            height: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let mut scrollbar_layout = crate::element::LayoutStyle::default();
                    scrollbar_layout.size.width = Length::Fill;
                    scrollbar_layout.size.height = Length::Fill;
                    let scrollbar = cx.scrollbar(crate::element::ScrollbarProps {
                        layout: scrollbar_layout,
                        axis: crate::element::ScrollbarAxis::Vertical,
                        scroll_target: None,
                        scroll_handle: crate::scroll::ScrollHandle::default(),
                        style: crate::element::ScrollbarStyle::default(),
                    });

                    vec![
                        cx.container(viewport, |_cx| Vec::<AnyElement>::new()),
                        cx.interactivity_gate_props(
                            crate::element::InteractivityGateProps {
                                layout: gate_layout,
                                present: false,
                                interactive: false,
                            },
                            move |cx| vec![cx.opacity(0.0, |_cx| vec![scrollbar])],
                        ),
                    ]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let stack_node = ui.children(child)[0];
    let gate_node = ui.children(stack_node)[1];
    assert!(
        ui.debug_node_measured_size(gate_node)
            .is_some_and(|size| size == Size::default()),
        "absent overlay gate should legitimately have a zero measured size"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "absent zero-size overlay chrome should not force the Stack root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "a legal absent 0x0 overlay should not be reported as missing_measured_size"
    );

    let gate_bounds = ui.debug_node_bounds(gate_node).expect("gate bounds");
    assert!((gate_bounds.origin.x.0 - rect_b.size.width.0).abs() < 0.01);
    assert!((gate_bounds.size.width.0 - 0.0).abs() < 0.01);
    assert!((gate_bounds.size.height.0 - 0.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_explicit_zero_spacer_leaf() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut text = FakeTextService::default();
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(216.0), Px(180.0)),
    );

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-explicit-zero-spacer",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| {
                    let mut layout = crate::element::LayoutStyle::default();
                    layout.size.width = Length::Px(Px(0.0));
                    layout.size.height = Length::Px(Px(0.0));
                    layout.flex.grow = 0.0;
                    layout.flex.shrink = 0.0;
                    layout.flex.basis = Length::Px(Px(0.0));
                    vec![cx.spacer(crate::element::SpacerProps {
                        layout,
                        min: Px(0.0),
                    })]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let stack_node = ui.children(child)[0];
    let spacer_node = ui.children(stack_node)[0];
    assert!(
        ui.debug_node_measured_size(spacer_node)
            .is_some_and(|size| size == Size::default()),
        "an explicit driver-only spacer can legitimately measure to zero"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "an explicit zero-size spacer leaf should not force the Stack root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "the explicit zero-size spacer leaf should not be reported as missing_measured_size"
    );

    let spacer_bounds = ui.debug_node_bounds(spacer_node).expect("spacer bounds");
    assert!((spacer_bounds.size.width.0 - 0.0).abs() < 0.01);
    assert!((spacer_bounds.size.height.0 - 0.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_rejects_implicit_zero_spacer_leaf() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut text = FakeTextService::default();
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(216.0), Px(180.0)),
    );

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-implicit-zero-spacer",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| vec![cx.spacer(crate::element::SpacerProps::default())],
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "implicit zero-size spacers should stay on the authoritative solve path until their intent is explicit"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("missing_measured_size")
    );
}

#[test]
fn clean_geometry_small_resize_skips_explicit_zero_container_leaf() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut text = FakeTextService::default();
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(216.0), Px(180.0)),
    );

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-explicit-zero-container",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| {
                    let mut props = crate::element::ContainerProps::default();
                    props.layout.size.width = Length::Px(Px(0.0));
                    props.layout.size.height = Length::Px(Px(0.0));
                    vec![cx.container(props, |_cx| Vec::new())]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let stack_node = ui.children(child)[0];
    let container_node = ui.children(stack_node)[0];
    assert!(
        ui.debug_node_measured_size(container_node)
            .is_some_and(|size| size == Size::default()),
        "an explicit driver-only container can legitimately measure to zero"
    );

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "an explicit zero-size container leaf should not force the Stack root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "the explicit zero-size container leaf should not be reported as missing_measured_size"
    );

    let container_bounds = ui
        .debug_node_bounds(container_node)
        .expect("container bounds");
    assert!((container_bounds.size.width.0 - 0.0).abs() < 0.01);
    assert!((container_bounds.size.height.0 - 0.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_rejects_implicit_zero_container_leaf() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    let window = AppWindowId::default();
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    let mut text = FakeTextService::default();
    let bounds_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(220.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(216.0), Px(180.0)),
    );

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-implicit-zero-container",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| vec![cx.container(Default::default(), |_cx| Vec::new())],
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "implicit zero-size containers should stay on the authoritative solve path until their intent is explicit"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("missing_measured_size")
    );
}

#[test]
fn clean_geometry_small_resize_rejects_fraction_absolute_stack_overlay_inset() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-fraction-absolute-overlay-child",
        |cx| {
            let stack = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };
            vec![cx.stack_props(stack, |cx| {
                let mut viewport = crate::element::ContainerProps::default();
                viewport.layout.size.width = Length::Fill;
                viewport.layout.size.height = Length::Fill;

                let gate_layout = crate::element::LayoutStyle {
                    position: crate::element::PositionStyle::Absolute,
                    inset: crate::element::InsetStyle {
                        top: Some(Px(0.0)).into(),
                        right: crate::element::InsetEdge::Fraction(0.5),
                        bottom: Some(Px(0.0)).into(),
                        left: None.into(),
                    },
                    size: crate::element::SizeStyle {
                        width: Length::Px(Px(8.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                vec![
                    cx.container(viewport, |_cx| Vec::<AnyElement>::new()),
                    cx.interactivity_gate_props(
                        crate::element::InteractivityGateProps {
                            layout: gate_layout,
                            present: true,
                            interactive: true,
                        },
                        |_cx| Vec::<AnyElement>::new(),
                    ),
                ]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "fraction inset absolute children must keep the authoritative root solve until the percent basis is proven"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("non_px_spacing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Stack")
    );
}

#[test]
fn clean_geometry_small_resize_propagates_to_view_cache_boundary_without_root_solve() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    ui.set_view_cache_enabled(true);

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-small-resize-view-cache-child",
        |cx| {
            let mut cache = crate::element::ViewCacheProps::default();
            cache.layout.size.width = Length::Fill;
            cache.layout.size.height = Length::Fill;
            cache = cache.contain_layout_when_bounds_known(true);
            vec![cx.view_cache(cache, |cx| {
                vec![cx.stack_props(
                    crate::element::StackProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    let cache_node = ui.children(child)[0];
    assert!(ui.should_reuse_view_cache_node(cache_node));

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "clean width-only resize should propagate to a clean ViewCache boundary without a Taffy root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted ViewCache boundary propagation should not leave rejection noise in frame stats"
    );
    assert_eq!(
        ui.debug_stats().view_cache_contained_relayouts,
        0,
        "clean cache roots should not be forced into contained relayout by parent geometry propagation"
    );

    let cache_bounds = ui.debug_node_bounds(cache_node).expect("cache bounds");
    assert!(
        (cache_bounds.size.width.0 - rect_b.size.width.0).abs() < 0.01,
        "ViewCache bounds must track the propagated root width"
    );

    assert!(
        ui.should_reuse_view_cache_node(cache_node),
        "clean geometry propagation must not mark the cache root for declarative rerender"
    );
    assert!(
        !ui.view_cache_node_needs_rerender(cache_node),
        "clean geometry propagation must leave view-cache rerender pressure unchanged"
    );
}

#[test]
fn clean_geometry_small_resize_keeps_view_cache_root_solve_as_boundary() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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
    ui.set_view_cache_enabled(true);

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let cache = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-view-cache-root-boundary-child",
        |cx| {
            let mut cache = crate::element::ViewCacheProps::default();
            cache.layout.size.width = Length::Fill;
            cache.layout.size.height = Length::Fill;
            cache = cache.contain_layout_when_bounds_known(true);
            let element = cx.view_cache(cache, |cx| {
                vec![cx.stack_props(
                    crate::element::StackProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                    |_cx| Vec::<AnyElement>::new(),
                )]
            });
            vec![element]
        },
    );
    let cache = ui.children(cache)[0];

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child: cache,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![cache]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "ViewCache as the explicit root remains a layout/cache boundary and should not skip its own solve"
    );
    assert!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections > 0,
        "boundary roots should report why their own root solve was kept"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("side_effect_boundary")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("ViewCache")
    );
}

#[test]
fn clean_geometry_rejection_reports_descendant_node_attribution() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-descendant-rejection-child",
        |cx| {
            let mut canvas = crate::element::CanvasProps::default();
            canvas.layout.size.width = Length::Fill;
            canvas.layout.size.height = Length::Fill;
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                |cx| {
                    vec![
                        cx.canvas(canvas, |_paint| {})
                            .test_id("clean-geometry-rejected-canvas"),
                    ]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let rejected_canvas = ui
        .debug_node_children(child)
        .into_iter()
        .next()
        .and_then(|stack| ui.debug_node_children(stack).into_iter().next())
        .expect("canvas descendant should be mounted");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    let rejected_solve = ui
        .debug_layout_engine_solves()
        .iter()
        .find(|solve| solve.root == child)
        .expect("child root solve should be recorded");
    let rejection = rejected_solve
        .clean_geometry_solve_skip_rejection
        .as_ref()
        .expect("child root solve should expose rejection details");

    assert_eq!(rejection.reason, "unsupported_kind");
    assert_eq!(rejection.element_kind, Some("Canvas"));
    assert_eq!(
        rejection.node,
        Some(rejected_canvas),
        "descendant rejections should report the actual rejected node, not just the solve root"
    );
    assert!(
        rejection.element.is_some(),
        "descendant rejection attribution should expose the rejected element id"
    );
    assert_eq!(
        rejection.element,
        ui.debug_node_element(rejected_canvas),
        "rejection element should match the rejected descendant node"
    );
}

#[test]
fn clean_geometry_small_resize_skips_px_container_and_updates_child_bounds() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-px-container-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;
            container.padding = crate::element::SpacingEdges {
                left: crate::element::SpacingLength::Px(Px(7.0)),
                right: crate::element::SpacingLength::Px(Px(11.0)),
                top: crate::element::SpacingLength::Px(Px(3.0)),
                bottom: crate::element::SpacingLength::Px(Px(5.0)),
            };
            container.border = fret_core::Edges {
                left: Px(2.0),
                right: Px(4.0),
                top: Px(1.0),
                bottom: Px(6.0),
            };
            let mut stack = crate::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Px(Px(20.0));
            vec![cx.container(container, |cx| {
                vec![cx.stack_props(stack, |_cx| Vec::<AnyElement>::new())]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "px-only Container geometry should not force a small width-delta root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted Container geometry skips should not report rejection noise"
    );

    let container_node = ui.children(child)[0];
    let stack_node = ui.children(container_node)[0];
    let stack_bounds = ui.debug_node_bounds(stack_node).expect("stack bounds");

    assert_eq!(
        stack_bounds.origin,
        Point::new(Px(9.0), Px(4.0)),
        "child origin should include px padding plus nonnegative border insets"
    );
    assert!(
        (stack_bounds.size.width.0 - 160.0).abs() < 0.01,
        "fill-width child should track the next Container content width"
    );
    assert!((stack_bounds.size.height.0 - 20.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_stable_auto_height_container_wrapper() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-stable-auto-height-container-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;

            let mut auto_container = crate::element::ContainerProps::default();
            auto_container.layout.size.width = Length::Fill;
            auto_container.layout.size.height = Length::Auto;

            let mut stack = crate::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Px(Px(20.0));

            vec![cx.container(container, |cx| {
                vec![cx.container(auto_container, |cx| {
                    vec![cx.stack_props(stack, |cx| {
                        vec![cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Px(Px(20.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(20.0),
                        })]
                    })]
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "stable auto-height wrappers should not force a small width-delta root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted stable auto-height wrappers should not report rejection noise"
    );

    let container_node = ui.children(child)[0];
    let auto_container_node = ui.children(container_node)[0];
    let stack_node = ui.children(auto_container_node)[0];
    let spacer_node = ui.children(stack_node)[0];
    let auto_container_bounds = ui
        .debug_node_bounds(auto_container_node)
        .expect("auto container bounds");
    let stack_bounds = ui.debug_node_bounds(stack_node).expect("stack bounds");
    let spacer_bounds = ui.debug_node_bounds(spacer_node).expect("spacer bounds");

    assert!((auto_container_bounds.size.width.0 - 184.0).abs() < 0.01);
    assert!((auto_container_bounds.size.height.0 - 20.0).abs() < 0.01);
    assert!((stack_bounds.size.width.0 - 184.0).abs() < 0.01);
    assert!((stack_bounds.size.height.0 - 20.0).abs() < 0.01);
    assert!((spacer_bounds.size.width.0 - 184.0).abs() < 0.01);
    assert!((spacer_bounds.size.height.0 - 20.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_stable_auto_height_vertical_flex_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-stable-auto-height-flex-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                align: crate::element::CrossAlign::Stretch,
                gap: Px(2.0).into(),
                ..Default::default()
            };

            let mut auto_container = crate::element::ContainerProps::default();
            auto_container.layout.size.width = Length::Fill;
            auto_container.layout.size.height = Length::Auto;

            let mut stack = crate::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Px(Px(20.0));

            vec![cx.flex(flex, |cx| {
                vec![cx.container(auto_container, |cx| {
                    vec![cx.stack_props(stack, |_cx| Vec::<AnyElement>::new())]
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "stable auto-height children in a vertical no-wrap flex should not force a small width-delta root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted stable auto-height flex children should not report rejection noise"
    );

    let flex_node = ui.children(child)[0];
    let auto_container_node = ui.children(flex_node)[0];
    let stack_node = ui.children(auto_container_node)[0];
    let auto_container_bounds = ui
        .debug_node_bounds(auto_container_node)
        .expect("auto container bounds");
    let stack_bounds = ui.debug_node_bounds(stack_node).expect("stack bounds");

    assert!((auto_container_bounds.size.width.0 - 184.0).abs() < 0.01);
    assert!((auto_container_bounds.size.height.0 - 20.0).abs() < 0.01);
    assert!((stack_bounds.size.width.0 - 184.0).abs() < 0.01);
    assert!((stack_bounds.size.height.0 - 20.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_rejects_center_aligned_vertical_flex_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-center-vertical-flex-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Vertical,
                align: crate::element::CrossAlign::Center,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(24.0)),
                            height: Length::Px(Px(18.0)),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(24.0),
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "center-aligned vertical flex children need a dedicated cross-axis proof"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_cross_align")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_skips_fixed_horizontal_flex_children() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-fixed-horizontal-flex-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Stretch,
                gap: Px(3.0).into(),
                padding: crate::element::SpacingEdges {
                    left: Px(4.0).into(),
                    right: Px(6.0).into(),
                    top: Px(2.0).into(),
                    bottom: Px(5.0).into(),
                },
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                [24.0, 32.0]
                    .into_iter()
                    .map(|width| {
                        cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(width)),
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(width),
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "fixed horizontal no-wrap flex children should not force a small width-delta root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted horizontal flex geometry skips should not report rejection noise"
    );

    let flex_node = ui.children(child)[0];
    let first_child = ui.children(flex_node)[0];
    let second_child = ui.children(flex_node)[1];
    let first_bounds = ui
        .debug_node_bounds(first_child)
        .expect("first child bounds");
    let second_bounds = ui
        .debug_node_bounds(second_child)
        .expect("second child bounds");

    assert_eq!(first_bounds.origin, Point::new(Px(4.0), Px(2.0)));
    assert!((first_bounds.size.width.0 - 24.0).abs() < 0.01);
    assert!((first_bounds.size.height.0 - 133.0).abs() < 0.01);
    assert_eq!(second_bounds.origin, Point::new(Px(31.0), Px(2.0)));
    assert!((second_bounds.size.width.0 - 32.0).abs() < 0.01);
    assert!((second_bounds.size.height.0 - 133.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_center_aligned_fixed_horizontal_flex_children() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-center-horizontal-flex-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Center,
                gap: Px(3.0).into(),
                padding: crate::element::SpacingEdges {
                    left: Px(4.0).into(),
                    right: Px(6.0).into(),
                    top: Px(2.0).into(),
                    bottom: Px(4.0).into(),
                },
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                [24.0, 32.0]
                    .into_iter()
                    .map(|width| {
                        cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(width)),
                                    height: Length::Px(Px(18.0)),
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(width),
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let flex_node = ui.children(child)[0];
    let first_child = ui.children(flex_node)[0];
    let first_bounds_before = ui
        .debug_node_bounds(first_child)
        .expect("first child bounds before resize");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "center-aligned fixed horizontal flex children should not force a small width-delta root solve; first rejection={:?}/{:?}",
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted center-aligned horizontal flex geometry skips should not report rejection noise"
    );

    let first_bounds_after = ui
        .debug_node_bounds(first_child)
        .expect("first child bounds after resize");
    assert_eq!(first_bounds_after.origin.y, first_bounds_before.origin.y);
    assert_eq!(
        first_bounds_after.size.height,
        first_bounds_before.size.height
    );
    assert!((first_bounds_after.size.width.0 - 24.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_center_justified_intrinsic_horizontal_flex() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-center-justified-intrinsic-horizontal-flex",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.height = Length::Fill;
            vec![cx.container(container, |cx| {
                let flex = crate::element::FlexProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            height: Length::Fill,
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: fret_core::Axis::Horizontal,
                    justify: crate::element::MainAlign::Center,
                    align: crate::element::CrossAlign::Center,
                    gap: Px(3.0).into(),
                    padding: crate::element::SpacingEdges {
                        left: Px(4.0).into(),
                        right: Px(6.0).into(),
                        top: Px(2.0).into(),
                        bottom: Px(4.0).into(),
                    },
                    ..Default::default()
                };

                vec![cx.flex(flex, |cx| {
                    [24.0, 32.0]
                        .into_iter()
                        .map(|width| {
                            cx.spacer(crate::element::SpacerProps {
                                layout: crate::element::LayoutStyle {
                                    size: crate::element::SizeStyle {
                                        width: Length::Px(Px(width)),
                                        height: Length::Px(Px(18.0)),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                min: Px(width),
                            })
                        })
                        .collect::<Vec<_>>()
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let container_node = ui.children(child)[0];
    let flex_node = ui.children(container_node)[0];
    let first_child = ui.children(flex_node)[0];
    let flex_before = ui.debug_node_bounds(flex_node).expect("flex before");
    let first_before = ui
        .debug_node_bounds(first_child)
        .expect("first child before");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "center-justified intrinsic horizontal flex should keep clean geometry when its own inner width is unchanged; first rejection={:?}/{:?}",
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0
    );

    let flex_after = ui.debug_node_bounds(flex_node).expect("flex after");
    let first_after = ui
        .debug_node_bounds(first_child)
        .expect("first child after");
    assert_eq!(flex_after.size.width, flex_before.size.width);
    assert_eq!(first_after, first_before);
}

#[test]
fn clean_geometry_small_resize_rejects_center_justified_fill_horizontal_flex_width_delta() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-center-justified-fill-horizontal-flex",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                justify: crate::element::MainAlign::Center,
                align: crate::element::CrossAlign::Center,
                gap: Px(3.0).into(),
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                [24.0, 32.0]
                    .into_iter()
                    .map(|width| {
                        cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(width)),
                                    height: Length::Px(Px(18.0)),
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(width),
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "center-justified fill horizontal flex changes free-space distribution under width deltas"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_main_align")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_skips_horizontal_flex_single_basis0_grow_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-single-grow-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Stretch,
                gap: Px(4.0).into(),
                padding: crate::element::SpacingEdges {
                    left: Px(6.0).into(),
                    right: Px(8.0).into(),
                    top: Px(2.0).into(),
                    bottom: Px(4.0).into(),
                },
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                let fixed = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(48.0)),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(48.0),
                });
                let grow = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_width: Some(Length::Px(Px(0.0))),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(0.0),
                });
                let trailing = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(16.0)),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(16.0),
                });
                vec![fixed, grow, trailing]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );
    let expected_delta = rect_b.size.width.0 - rect_a.size.width.0;

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let flex_node = ui.children(child)[0];
    let fixed_child = ui.children(flex_node)[0];
    let grow_child = ui.children(flex_node)[1];
    let trailing_child = ui.children(flex_node)[2];
    let fixed_before = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed child bounds before resize");
    let grow_before = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds before resize");
    let trailing_before = ui
        .debug_node_bounds(trailing_child)
        .expect("trailing child bounds before resize");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "single basis-0 grow child in a horizontal flex row should absorb a small width delta without a root solve; first rejection={:?}/{:?}",
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted basis-0 grow horizontal flex geometry skips should not report rejection noise"
    );

    let fixed_after = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed child bounds after resize");
    let grow_after = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds after resize");
    let trailing_after = ui
        .debug_node_bounds(trailing_child)
        .expect("trailing child bounds after resize");
    assert_eq!(fixed_after, fixed_before);
    assert_eq!(grow_after.origin, grow_before.origin);
    assert_eq!(grow_after.size.height, grow_before.size.height);
    assert!((grow_after.size.width.0 - (grow_before.size.width.0 + expected_delta)).abs() < 0.01);
    assert!(
        (trailing_after.origin.x.0 - (trailing_before.origin.x.0 + expected_delta)).abs() < 0.01
    );
    assert_eq!(trailing_after.origin.y, trailing_before.origin.y);
    assert_eq!(trailing_after.size, trailing_before.size);
}

#[test]
fn clean_geometry_small_resize_rejects_horizontal_flex_multiple_grow_children() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-multiple-grow-children",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Stretch,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                (0..2)
                    .map(|_| {
                        cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    min_width: Some(Length::Px(Px(0.0))),
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    grow: 1.0,
                                    shrink: 1.0,
                                    basis: Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(0.0),
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "multiple horizontal flex grow children need a dedicated distribution proof"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_item_sizing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_rejects_horizontal_flex_fixed_px_default_shrink_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-fixed-px-default-shrink-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Stretch,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                let fixed_default_shrink = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(48.0)),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(48.0),
                });
                let grow = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_width: Some(Length::Px(Px(0.0))),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(0.0),
                });
                vec![fixed_default_shrink, grow]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "fixed px children with default flex-shrink still require flex distribution proof"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_item_sizing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_skips_horizontal_flex_empty_grow_container_slot() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-empty-grow-container-slot",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                gap: Px(8.0).into(),
                align: crate::element::CrossAlign::Center,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                let fixed = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(32.0)),
                            height: Length::Px(Px(16.0)),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(0.0),
                });
                let grow = cx.hit_test_gate_props(
                    crate::element::HitTestGateProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                min_width: Some(Length::Px(Px(0.0))),
                                ..Default::default()
                            },
                            flex: crate::element::FlexItemStyle {
                                grow: 1.0,
                                shrink: 1.0,
                                basis: Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        hit_test: false,
                    },
                    |_cx| Vec::<AnyElement>::new(),
                );
                vec![fixed, grow]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let flex_node = ui.children(child)[0];
    let grow_node = ui.children(flex_node)[1];
    let grow_before = ui.debug_node_bounds(grow_node).expect("grow bounds before");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "single empty grow container slot should keep the app-shell flex path in clean geometry propagation"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "the empty grow slot must not trip the missing_measured_size spacer sentinel"
    );
    let grow_after = ui.debug_node_bounds(grow_node).expect("grow bounds after");
    assert_eq!(grow_after.origin, grow_before.origin);
    assert_eq!(grow_after.size.height, grow_before.size.height);
    assert!((grow_after.size.width.0 - (grow_before.size.width.0 - 4.0)).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_horizontal_flex_auto_width_no_shrink_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-auto-width-no-shrink-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Center,
                gap: Px(4.0).into(),
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                let fixed_auto = cx.container(
                    crate::element::ContainerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Auto,
                                height: Length::Px(Px(20.0)),
                                min_width: Some(Length::Px(Px(24.0))),
                                ..Default::default()
                            },
                            flex: crate::element::FlexItemStyle {
                                shrink: 0.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(48.0)),
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(48.0),
                        })]
                    },
                );
                let grow = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_width: Some(Length::Px(Px(0.0))),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(0.0),
                });
                vec![fixed_auto, grow]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );
    let expected_delta = rect_b.size.width.0 - rect_a.size.width.0;

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let flex_node = ui.children(child)[0];
    let fixed_child = ui.children(flex_node)[0];
    let grow_child = ui.children(flex_node)[1];
    let fixed_before = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed auto-width child bounds before resize");
    let grow_before = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds before resize");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "auto-width no-shrink horizontal flex item should keep its computed width while the basis-0 grow child absorbs the delta; first rejection={:?}/{:?}",
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted auto-width no-shrink horizontal flex geometry skips should not report rejection noise"
    );

    let fixed_after = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed auto-width child bounds after resize");
    let grow_after = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds after resize");
    assert_eq!(fixed_after, fixed_before);
    assert!((grow_after.size.width.0 - (grow_before.size.width.0 + expected_delta)).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_horizontal_roving_flex_auto_width_no_shrink_child() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-roving-flex-auto-width-no-shrink-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Center,
                gap: Px(4.0).into(),
                ..Default::default()
            };

            vec![cx.roving_flex(
                crate::element::RovingFlexProps {
                    flex,
                    ..Default::default()
                },
                |cx| {
                    let fixed_auto = cx.container(
                        crate::element::ContainerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Auto,
                                    height: Length::Px(Px(20.0)),
                                    min_width: Some(Length::Px(Px(24.0))),
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx| {
                            vec![cx.spacer(crate::element::SpacerProps {
                                layout: crate::element::LayoutStyle {
                                    size: crate::element::SizeStyle {
                                        width: Length::Px(Px(48.0)),
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    flex: crate::element::FlexItemStyle {
                                        shrink: 0.0,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                min: Px(48.0),
                            })]
                        },
                    );
                    let grow = cx.spacer(crate::element::SpacerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                min_width: Some(Length::Px(Px(0.0))),
                                ..Default::default()
                            },
                            flex: crate::element::FlexItemStyle {
                                grow: 1.0,
                                shrink: 1.0,
                                basis: Length::Px(Px(0.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        min: Px(0.0),
                    });
                    vec![fixed_auto, grow]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );
    let expected_delta = rect_b.size.width.0 - rect_a.size.width.0;

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    let roving_flex_node = ui.children(child)[0];
    let fixed_child = ui.children(roving_flex_node)[0];
    let grow_child = ui.children(roving_flex_node)[1];
    let fixed_before = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed auto-width child bounds before resize");
    let grow_before = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds before resize");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "RovingFlex should use the same clean horizontal flex proof as Flex for auto-width no-shrink items; first rejection={:?}/{:?}",
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted RovingFlex geometry skips should not report rejection noise"
    );

    let fixed_after = ui
        .debug_node_bounds(fixed_child)
        .expect("fixed auto-width child bounds after resize");
    let grow_after = ui
        .debug_node_bounds(grow_child)
        .expect("grow child bounds after resize");
    assert_eq!(fixed_after, fixed_before);
    assert!((grow_after.size.width.0 - (grow_before.size.width.0 + expected_delta)).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_rejects_horizontal_flex_auto_width_child_fractional_max_constraint()
{
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(316.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-auto-width-child-fractional-max",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                let fixed_auto = cx.container(
                    crate::element::ContainerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Auto,
                                height: Length::Px(Px(20.0)),
                                max_width: Some(Length::Fraction(0.5)),
                                ..Default::default()
                            },
                            flex: crate::element::FlexItemStyle {
                                shrink: 0.0,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(48.0)),
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                flex: crate::element::FlexItemStyle {
                                    shrink: 0.0,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(48.0),
                        })]
                    },
                );
                let grow = cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_width: Some(Length::Px(Px(0.0))),
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(0.0),
                });
                vec![fixed_auto, grow]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(176.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "auto-width fixed flex children must reject when max constraints need a parent-width basis"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_item_sizing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_rejects_horizontal_flex_grow_children() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-horizontal-flex-grow-child",
        |cx| {
            let flex = crate::element::FlexProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: fret_core::Axis::Horizontal,
                align: crate::element::CrossAlign::Stretch,
                ..Default::default()
            };

            vec![cx.flex(flex, |cx| {
                vec![cx.spacer(crate::element::SpacerProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Px(Px(24.0)),
                            height: Length::Fill,
                            ..Default::default()
                        },
                        flex: crate::element::FlexItemStyle {
                            grow: 1.0,
                            shrink: 0.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    min: Px(24.0),
                })]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "horizontal flex grow children need a dedicated main-axis distribution proof"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_item_sizing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_rejects_auto_height_text_reflow() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-auto-height-text-reflow-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;

            let mut auto_container = crate::element::ContainerProps::default();
            auto_container.layout.size.width = Length::Fill;
            auto_container.layout.size.height = Length::Auto;

            let mut text_props = crate::element::TextProps::new(
                "width dependent text should keep the authoritative solve when its box changes",
            );
            text_props.layout.size.width = Length::Fill;
            text_props.layout.size.height = Length::Auto;

            vec![cx.container(container, |cx| {
                vec![cx.container(auto_container, |cx| vec![cx.text_props(text_props)])]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "auto-height text whose width changes must keep the authoritative solve"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("text_reflow")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Text")
    );
    let rejected_solve = ui
        .debug_layout_engine_solves()
        .iter()
        .find(|solve| solve.root == child)
        .expect("child root solve should be recorded");
    let rejection = rejected_solve
        .clean_geometry_solve_skip_rejection
        .as_ref()
        .expect("child root solve should expose text rejection details");

    assert_eq!(rejection.reason, "text_reflow");
    assert_eq!(rejection.detail, Some("text_wrap_not_none"));
    assert_eq!(rejection.element_kind, Some("Text"));
}

#[test]
fn clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-nowrap-text-width-delta-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;

            let mut text_props = crate::element::TextProps::new("nowrap text stays single-line");
            text_props.wrap = fret_core::TextWrap::None;
            text_props.layout.size.width = Length::Auto;
            text_props.layout.size.height = Length::Auto;

            vec![cx.container(container, |cx| {
                vec![
                    cx.text_props(text_props)
                        .test_id("clean-geometry-nowrap-stable-text"),
                ]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);
    let container_node = ui.children(child)[0];
    let text_node = ui.children(container_node)[0];
    let text_bounds_before = ui.debug_node_bounds(text_node).expect("text bounds before");
    let text_measured_before = ui
        .debug_node_measured_size(text_node)
        .expect("text measured size before");

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "nowrap text with stable height should not force an authoritative engine solve"
    );
    assert_eq!(
        ui.debug_node_bounds(text_node).expect("text bounds after"),
        text_bounds_before,
        "auto-sized nowrap text keeps its natural computed box across parent width deltas"
    );
    assert_eq!(
        ui.debug_node_measured_size(text_node)
            .expect("text measured size after"),
        text_measured_before,
        "cached nowrap text metrics should remain the measured size after clean propagation"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        None
    );
}

#[test]
fn clean_geometry_small_resize_rejects_nowrap_text_height_delta() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-nowrap-text-height-delta-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;

            let mut text_props = crate::element::TextProps::new("nowrap text height guard");
            text_props.wrap = fret_core::TextWrap::None;
            text_props.layout.size.width = Length::Fill;
            text_props.layout.size.height = Length::Fill;

            vec![cx.container(container, |cx| vec![cx.text_props(text_props)])]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(144.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "text height deltas need the authoritative solve even when wrapping is disabled"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("height_delta")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        None
    );
}

#[test]
fn clean_geometry_small_resize_rejects_container_fraction_padding() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-container-fraction-padding-child",
        |cx| {
            let mut container = crate::element::ContainerProps::default();
            container.layout.size.width = Length::Fill;
            container.layout.size.height = Length::Fill;
            container.padding.left = crate::element::SpacingLength::Fraction(0.1);
            let mut stack = crate::element::StackProps::default();
            stack.layout.size.width = Length::Fill;
            stack.layout.size.height = Length::Px(Px(20.0));
            vec![cx.container(container, |cx| {
                vec![cx.stack_props(stack, |_cx| Vec::<AnyElement>::new())]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "fraction padding must keep the authoritative root solve until percent basis propagation is proven"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("non_px_spacing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Container")
    );
}

#[test]
fn clean_geometry_small_resize_reports_wrap_flex_rejection_reason() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-wrap-flex-rejection-child",
        |cx| {
            let flex = crate::element::FlexProps {
                direction: fret_core::Axis::Vertical,
                wrap: true,
                gap: Px(1.0).into(),
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
                (0..8)
                    .map(|_| {
                        cx.spacer(crate::element::SpacerProps {
                            layout: crate::element::LayoutStyle {
                                size: crate::element::SizeStyle {
                                    width: Length::Px(Px(90.0)),
                                    height: Length::Px(Px(8.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            min: Px(8.0),
                        })
                    })
                    .collect::<Vec<_>>()
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "wrapped flex must keep the authoritative engine solve until line-break stability is proven"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("flex_wrap")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Flex")
    );
}

#[test]
fn clean_geometry_small_resize_preserves_fixed_stack_child_width() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-fixed-stack-child",
        |cx| {
            vec![cx.stack_props(
                crate::element::StackProps {
                    layout: crate::element::LayoutStyle {
                        size: crate::element::SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                move |cx| {
                    vec![cx.spacer(crate::element::SpacerProps {
                        layout: crate::element::LayoutStyle {
                            size: crate::element::SizeStyle {
                                width: Length::Px(Px(24.0)),
                                height: Length::Px(Px(12.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        min: Px(24.0),
                    })]
                },
            )]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "fixed Stack child should still allow the clean root solve skip"
    );
    let stack_node = ui.children(child)[0];
    let spacer_node = ui.children(stack_node)[0];
    let spacer_bounds = ui.debug_node_bounds(spacer_node).expect("spacer bounds");
    assert!((spacer_bounds.size.width.0 - 24.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_skips_card_header_like_auto_grid() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-card-header-like-grid-child",
        |cx| {
            let grid = crate::element::GridProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cols: 1,
                rows: Some(2),
                template_rows: Some(vec![
                    crate::element::GridTrackSizing::Auto,
                    crate::element::GridTrackSizing::Auto,
                ]),
                gap: Px(3.0).into(),
                padding: crate::element::SpacingEdges {
                    left: crate::element::SpacingLength::Px(Px(7.0)),
                    right: crate::element::SpacingLength::Px(Px(11.0)),
                    top: crate::element::SpacingLength::Px(Px(5.0)),
                    bottom: crate::element::SpacingLength::Px(Px(13.0)),
                },
                align: crate::element::CrossAlign::Start,
                ..Default::default()
            };

            let title = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(Px(14.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };
            let description = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(Px(9.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };

            vec![cx.grid(grid, |cx| {
                vec![
                    cx.stack_props(title, |_cx| Vec::<AnyElement>::new()),
                    cx.stack_props(description, |_cx| Vec::<AnyElement>::new()),
                ]
            })]
        },
    );

    let rect_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(180.0), Px(44.0)));
    let rect_b = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(184.0), Px(44.0)));

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert_eq!(
        ui.debug_stats().layout_engine_solves,
        0,
        "single-column auto-row Grid geometry should not force a small width-delta root solve"
    );
    assert_eq!(
        ui.debug_stats().layout_clean_geometry_solve_skip_rejections,
        0,
        "accepted Grid geometry skips should not report rejection noise"
    );

    let grid_node = ui.children(child)[0];
    let rows = ui.children(grid_node);
    let title_bounds = ui.debug_node_bounds(rows[0]).expect("title bounds");
    let description_bounds = ui.debug_node_bounds(rows[1]).expect("description bounds");

    assert_eq!(title_bounds.origin, Point::new(Px(7.0), Px(5.0)));
    assert!((title_bounds.size.width.0 - 166.0).abs() < 0.01);
    assert!((title_bounds.size.height.0 - 14.0).abs() < 0.01);
    assert_eq!(description_bounds.origin, Point::new(Px(7.0), Px(22.0)));
    assert!((description_bounds.size.width.0 - 166.0).abs() < 0.01);
    assert!((description_bounds.size.height.0 - 9.0).abs() < 0.01);
}

#[test]
fn clean_geometry_small_resize_rejects_flexible_grid_track() {
    struct PrecomputeThenResize {
        child: NodeId,
        rect_a: Rect,
        rect_b: Rect,
        calls: u32,
    }

    impl<H: UiHost> Widget<H> for PrecomputeThenResize {
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

    let bounds_a = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );
    let bounds_b = Rect::new(
        fret_core::Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(324.0), Px(180.0)),
    );
    let mut text = FakeTextService::default();

    let child = render_root(
        &mut ui,
        &mut app,
        &mut text,
        window,
        bounds_a,
        "clean-geometry-flexible-grid-track-child",
        |cx| {
            let grid = crate::element::GridProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cols: 1,
                rows: Some(1),
                template_columns: Some(vec![crate::element::GridTrackSizing::Flex(1.0)]),
                template_rows: Some(vec![crate::element::GridTrackSizing::Auto]),
                align: crate::element::CrossAlign::Start,
                ..Default::default()
            };
            let child = crate::element::StackProps {
                layout: crate::element::LayoutStyle {
                    size: crate::element::SizeStyle {
                        width: Length::Fill,
                        height: Length::Px(Px(14.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            };

            vec![cx.grid(grid, |cx| {
                vec![cx.stack_props(child, |_cx| Vec::<AnyElement>::new())]
            })]
        },
    );

    let rect_a = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(180.0), Px(140.0)),
    );
    let rect_b = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(184.0), Px(140.0)),
    );

    let parent = ui.create_node(PrecomputeThenResize {
        child,
        rect_a,
        rect_b,
        calls: 0,
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    ui.layout_all(&mut app, &mut text, bounds_a, 1.0);

    app.advance_frame();
    ui.invalidate(parent, Invalidation::Layout);
    ui.layout_all(&mut app, &mut text, bounds_b, 1.0);

    assert!(
        ui.debug_stats().layout_engine_solves > 0,
        "flexible Grid tracks must keep the authoritative solve path"
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_rejection,
        Some("grid_track_sizing")
    );
    assert_eq!(
        ui.debug_stats()
            .layout_clean_geometry_solve_skip_first_element_kind,
        Some("Grid")
    );
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
                    gap: Px(8.0).into(),
                    padding: fret_core::Edges::all(Px(10.0)).into(),
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
            gap: Px(4.0).into(),
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
