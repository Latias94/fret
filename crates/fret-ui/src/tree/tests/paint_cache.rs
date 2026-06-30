use super::*;

#[test]
fn paint_cache_replays_ops_when_plain_node_translates_from_boundary_entry_store() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds_a = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 1);
    assert!(
        ui.test_paint_cache_entry_for_node_has_entry(node),
        "plain paint-cache roots should store replay entries in boundary paint-cache entry storage"
    );
    assert!(
        ui.test_retained_paint_cache_entry_store_has_entry(node),
        "plain paint-cache roots should use the retained entry store instead of becoming runtime ViewBoundaries"
    );
    assert!(
        !ui.test_view_boundary_exists(node),
        "plain paint-cache entries must not create full runtime ViewBoundary records"
    );

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    let bounds_b = Rect::new(
        Point::new(fret_core::Px(20.0), fret_core::Px(15.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.ops_len(), 3);

    match (scene.ops()[0], scene.ops()[1], scene.ops()[2]) {
        (
            SceneOp::PushTransform { transform },
            SceneOp::Quad { rect, .. },
            SceneOp::PopTransform,
        ) => {
            assert_eq!(transform.tx, bounds_b.origin.x.0 - bounds_a.origin.x.0);
            assert_eq!(transform.ty, bounds_b.origin.y.0 - bounds_a.origin.y.0);
            assert_eq!(rect, bounds_a);
        }
        _ => panic!("expected push-transform + quad + pop-transform ops"),
    }
}

#[test]
fn paint_cache_entry_is_boundary_owned_for_view_cache_roots() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_node_view_cache_flags(node, true, true, true);
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert!(
        ui.test_view_boundary_paint_cache_has_entry(node),
        "view-cache roots should store paint-cache entries in ViewBoundaryState"
    );
    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|stats| stats.id == node)
        .expect("boundary stats for view-cache root");
    assert_eq!(
        boundary.paint_cache_owner,
        "view_boundary_paint_cache_state"
    );

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        paints.load(Ordering::SeqCst),
        1,
        "boundary-owned paint cache should replay for clean view-cache roots"
    );
    assert_eq!(ui.debug_stats().paint_cache_hits, 1);
}

#[test]
fn paint_cache_side_store_entry_migrates_when_node_becomes_view_boundary() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert!(ui.test_retained_paint_cache_entry_store_has_entry(node));
    assert!(!ui.test_view_boundary_exists(node));

    ui.set_node_view_cache_flags(node, true, true, true);

    assert!(
        ui.test_view_boundary_paint_cache_has_entry(node),
        "promoting a cached plain node to a runtime boundary should migrate the replay entry"
    );
    assert!(
        !ui.test_retained_paint_cache_entry_store_has_entry(node),
        "promoted boundaries should not keep a duplicate retained entry"
    );

    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|stats| stats.id == node)
        .expect("boundary stats for promoted node");
    assert_eq!(
        boundary.paint_cache_owner,
        "view_boundary_paint_cache_state"
    );

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        paints.load(Ordering::SeqCst),
        1,
        "migrated boundary paint-cache entries should remain replayable"
    );
    assert_eq!(ui.debug_stats().paint_cache_hits, 1);
}

#[test]
fn window_paint_replay_ingests_scene_and_clears_when_recording_invalidates() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(scene.ops_len(), 1);
    assert_eq!(ui.test_retained_paint_recording_ops_len(), 0);

    ui.ingest_paint_cache_source(&mut scene);
    assert_eq!(
        ui.test_retained_paint_recording_ops_len(),
        1,
        "ingest should move scene ops into the retained previous-frame paint replay source"
    );

    ui.set_paint_cache_enabled(false);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        ui.test_retained_paint_recording_ops_len(),
        0,
        "disabling paint cache should clear the retained previous-frame paint replay source"
    );
}

#[test]
fn window_paint_replay_preserves_text_blob_side_index() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let text_blob = fret_core::TextBlobId::from(KeyData::from_ffi(101));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    struct TextPaintWidget {
        paints: Arc<AtomicUsize>,
        text_blob: fret_core::TextBlobId,
    }

    impl<H: UiHost> Widget<H> for TextPaintWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(0),
                origin: cx.bounds.origin,
                text: self.text_blob,
                paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                outline: None,
                shadow: None,
            });
        }
    }

    let node = ui.create_node(TextPaintWidget {
        paints: paints.clone(),
        text_blob,
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);
    assert_eq!(scene.text_blob_ids(), &[text_blob]);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        paints.load(Ordering::SeqCst),
        1,
        "text widget should be replayed from the previous-frame recording"
    );
    assert_eq!(
        scene.text_blob_ids(),
        &[text_blob],
        "paint-cache replay should preserve the precomputed text blob side index"
    );
}

#[test]
fn paint_cache_key_tracks_child_geometry_changes_when_parent_size_is_stable() {
    let mut app = crate::test_host::TestHost::new();

    let parent_paints = Arc::new(AtomicUsize::new(0));
    let child_paints = Arc::new(AtomicUsize::new(0));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    struct ParentWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for ParentWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });
            for &child in cx.children {
                if let Some(bounds) = cx.child_bounds(child) {
                    cx.paint(child, bounds);
                }
            }
        }
    }

    struct ChildWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for ChildWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });
        }
    }

    let parent = ui.create_node(ParentWidget {
        paints: parent_paints.clone(),
    });
    let child = ui.create_node(ChildWidget {
        paints: child_paints.clone(),
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    let parent_bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let child_bounds_a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(80.0), Px(10.0)));
    let child_bounds_b = Rect::new(Point::new(Px(0.0), Px(14.0)), Size::new(Px(60.0), Px(24.0)));

    ui.nodes.get_mut(parent).expect("parent").bounds = parent_bounds;
    ui.nodes.get_mut(parent).expect("parent").measured_size = parent_bounds.size;
    ui.nodes.get_mut(child).expect("child").bounds = child_bounds_a;
    ui.nodes.get_mut(child).expect("child").measured_size = child_bounds_a.size;
    ui.test_recompute_paint_geometry_fingerprint_subtree(parent);
    let parent_geometry_before = ui
        .nodes
        .get(parent)
        .expect("parent")
        .paint_geometry_fingerprint;
    let child_geometry_before = ui
        .nodes
        .get(child)
        .expect("child")
        .paint_geometry_fingerprint;

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, parent_bounds, &mut scene, 1.0);
    assert_eq!(parent_paints.load(Ordering::SeqCst), 1);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);
    ui.test_clear_node_invalidations(parent);
    ui.test_clear_node_invalidations(child);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.nodes.get_mut(child).expect("child").bounds = child_bounds_b;
    ui.nodes.get_mut(child).expect("child").measured_size = child_bounds_b.size;
    ui.test_recompute_paint_geometry_fingerprint_subtree(parent);
    ui.test_clear_node_invalidations(parent);
    ui.test_clear_node_invalidations(child);
    let parent_geometry_after = ui
        .nodes
        .get(parent)
        .expect("parent")
        .paint_geometry_fingerprint;
    let child_geometry_after = ui
        .nodes
        .get(child)
        .expect("child")
        .paint_geometry_fingerprint;
    assert_ne!(
        child_geometry_before, child_geometry_after,
        "child geometry fingerprint should change when the child layout changes"
    );
    assert_ne!(
        parent_geometry_before, parent_geometry_after,
        "parent geometry fingerprint should change when a descendant layout changes"
    );

    ui.paint_all(&mut app, &mut services, parent_bounds, &mut scene, 1.0);
    assert_eq!(
        parent_paints.load(Ordering::SeqCst),
        2,
        "stable parent bounds must not replay stale child geometry after a layout change"
    );
    assert_eq!(child_paints.load(Ordering::SeqCst), 2);
    assert!(
        ui.debug_stats().paint_cache_replayed_ops == 0,
        "child geometry fingerprint should force repaint rather than replay stale cached ops"
    );
}

#[test]
fn paint_cache_replay_translates_descendant_bounds_for_descendants() {
    let mut app = crate::test_host::TestHost::new();

    let parent_paints = Arc::new(AtomicUsize::new(0));
    let child_paints = Arc::new(AtomicUsize::new(0));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    struct ParentWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for ParentWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });

            let Some(&child) = cx.children.first() else {
                return;
            };
            let child_bounds = Rect::new(
                Point::new(
                    fret_core::Px(cx.bounds.origin.x.0 + 8.0),
                    fret_core::Px(cx.bounds.origin.y.0 + 6.0),
                ),
                Size::new(fret_core::Px(30.0), fret_core::Px(12.0)),
            );
            cx.paint(child, child_bounds);
        }
    }

    struct ChildWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for ChildWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });
        }
    }

    let parent = ui.create_node(ParentWidget {
        paints: parent_paints.clone(),
    });
    let child = ui.create_node(ChildWidget {
        paints: child_paints.clone(),
    });
    ui.set_children(parent, vec![child]);
    ui.set_root(parent);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds_a = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
    assert_eq!(parent_paints.load(Ordering::SeqCst), 1);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);

    let origin_a = ui.nodes.get(child).expect("child node").bounds.origin;

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    let bounds_b = Rect::new(
        Point::new(fret_core::Px(20.0), fret_core::Px(15.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);

    // Cache hit: parent/child paints are skipped and previous ops are replayed.
    assert_eq!(parent_paints.load(Ordering::SeqCst), 1);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);

    let delta = Point::new(
        bounds_b.origin.x - bounds_a.origin.x,
        bounds_b.origin.y - bounds_a.origin.y,
    );
    let origin_b = ui.nodes.get(child).expect("child node").bounds.origin;
    assert_eq!(
        origin_b,
        Point::new(origin_a.x + delta.x, origin_a.y + delta.y),
        "expected paint-cache replay to keep descendant bounds in sync with translated output"
    );
}

#[test]
fn paint_cache_replay_touches_selectable_text_span_state_for_replayed_subtrees() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let element = crate::elements::GlobalElementId(10_001);
    let mut ui = UiTree::new();
    ui.set_window(window);
    ui.set_paint_cache_enabled(true);
    ui.set_debug_enabled(true);

    struct SpanStateWidget {
        element: crate::elements::GlobalElementId,
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for SpanStateWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            let Some(window) = cx.window else {
                return;
            };
            crate::elements::with_element_state(
                cx.app,
                window,
                self.element,
                crate::element::SelectableTextState::default,
                |state| {
                    state.interactive_span_bounds =
                        vec![crate::element::SelectableTextInteractiveSpanBounds {
                            range: 6..10,
                            tag: Arc::<str>::from("https://example.com"),
                            bounds_local: Rect::new(
                                Point::new(Px(6.0), Px(0.0)),
                                Size::new(Px(4.0), Px(10.0)),
                            ),
                        }];
                },
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });
        }
    }

    let paints = Arc::new(AtomicUsize::new(0));
    let node = ui.create_node_for_element(
        element,
        SpanStateWidget {
            element,
            paints: paints.clone(),
        },
    );
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let moved_bounds = Rect::new(
        Point::new(Px(20.0), Px(15.0)),
        Size::new(Px(100.0), Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, moved_bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    for _ in 0..3 {
        app.advance_frame();
        ui.ingest_paint_cache_source(&mut scene);
        scene.clear();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
        assert_eq!(
            paints.load(Ordering::SeqCst),
            1,
            "expected paint-cache replay to skip widget paint"
        );
        assert_eq!(
            ui.debug_stats().paint_cache_hits,
            1,
            "expected paint-cache replay for the selectable state node"
        );
    }

    let spans = app.with_global_mut(crate::elements::ElementRuntime::new, |runtime, _| {
        runtime
            .selectable_text_interactive_span_bounds_for_element(window, element)
            .expect("selectable span state should survive repeated cache-hit frames")
    });

    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].range, 6..10);
    assert_eq!(spans[0].tag.as_ref(), "https://example.com");
    assert_eq!(spans[0].bounds_local.origin.x, Px(6.0));
    assert_eq!(spans[0].bounds_local.size.width, Px(4.0));
}

#[test]
fn paint_cache_rebases_descendant_entries_after_ancestor_replay() {
    let mut app = crate::test_host::TestHost::new();

    let child_paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let child = ui.create_node(CountingPaintWidget {
        paints: child_paints.clone(),
    });
    ui.set_children(root, vec![child]);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let moved_bounds = Rect::new(
        Point::new(Px(20.0), Px(15.0)),
        Size::new(Px(100.0), Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);
    assert!(
        ui.test_paint_cache_entry_for_node_has_entry(root),
        "root should cache the full subtree after the initial paint"
    );
    assert!(
        ui.test_paint_cache_entry_for_node_has_entry(child),
        "child should cache its local paint range after the initial paint"
    );

    app.advance_frame();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, moved_bounds, &mut scene, 1.0);
    assert_eq!(
        child_paints.load(Ordering::SeqCst),
        1,
        "second frame should replay the ancestor subtree"
    );
    assert!(
        ui.debug_paint_cache_replays.contains_key(&root),
        "second frame should hit the root cache"
    );
    assert!(
        !ui.debug_paint_cache_replays.contains_key(&child),
        "child is not visited when the ancestor cache replays"
    );

    app.advance_frame();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(root, Invalidation::Paint);
    ui.test_clear_node_invalidations(child);
    ui.paint_all(&mut app, &mut services, moved_bounds, &mut scene, 1.0);

    assert_eq!(
        child_paints.load(Ordering::SeqCst),
        1,
        "ancestor repaint should still be able to replay stable child entries after an ancestor-only replay frame"
    );
    assert!(
        !ui.debug_paint_cache_replays.contains_key(&root),
        "paint-invalidated root should not replay"
    );
    assert!(
        ui.debug_paint_cache_replays.contains_key(&child),
        "stable child should replay from the descendant entry rebased during the previous ancestor replay"
    );
}

#[test]
fn paint_cache_rebase_prunes_paint_invalidated_descendant_subtrees() {
    let mut app = crate::test_host::TestHost::new();

    let child_paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let mid = ui.create_node(TestStack);
    let child = ui.create_node(CountingPaintWidget {
        paints: child_paints.clone(),
    });
    ui.set_children(root, vec![mid]);
    ui.set_children(mid, vec![child]);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    let moved_bounds = Rect::new(
        Point::new(Px(20.0), Px(15.0)),
        Size::new(Px(100.0), Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);

    app.advance_frame();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.test_set_paint_invalidation(mid, true);
    ui.paint_all(&mut app, &mut services, moved_bounds, &mut scene, 1.0);
    assert_eq!(
        child_paints.load(Ordering::SeqCst),
        1,
        "second frame should still replay the clean ancestor subtree"
    );
    assert!(
        ui.debug_paint_cache_replays.contains_key(&root),
        "second frame should hit the root cache"
    );

    app.advance_frame();
    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(root, Invalidation::Paint);
    ui.test_clear_node_invalidations(child);
    ui.paint_all(&mut app, &mut services, moved_bounds, &mut scene, 1.0);

    assert_eq!(
        child_paints.load(Ordering::SeqCst),
        2,
        "a paint-invalidated intermediate node should prune descendant rebase so child repaint is not skipped"
    );
    assert!(
        !ui.debug_paint_cache_replays.contains_key(&root),
        "paint-invalidated root should not replay"
    );
    assert!(
        !ui.debug_paint_cache_replays.contains_key(&child),
        "child should not replay through a paint-invalidated ancestor"
    );
}

#[test]
fn paint_cache_does_not_replay_ops_when_widget_requests_animation_frame() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    struct RafWidget {
        paints: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for RafWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            self.paints.fetch_add(1, Ordering::SeqCst);
            cx.request_animation_frame();
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(0),
                rect: cx.bounds,
                background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                border: Edges::default(),
                border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
                corner_radii: Corners::default(),
            });
        }
    }

    let node = ui.create_node(RafWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(
        paints.load(Ordering::SeqCst),
        2,
        "expected a repaint after request_animation_frame, even with paint caching enabled"
    );
}

#[test]
fn paint_cache_is_cleared_when_caching_is_disabled_for_a_node() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let use_transform = Arc::new(AtomicBool::new(false));

    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(ToggleTransformPaintWidget {
        paints: paints.clone(),
        use_transform: use_transform.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    // Cache hit: paint is skipped and previous ops are replayed.
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    // Disable caching for the node (render transform present).
    use_transform.store(true, Ordering::SeqCst);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    // Re-enable caching: should *not* replay the stale cache entry from the pre-transform frame.
    use_transform.store(false, Ordering::SeqCst);
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 3);
}

#[test]
fn paint_cache_hit_test_only_invalidation_replays_when_cache_key_matches() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);
    ui.set_debug_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(node, Invalidation::HitTestOnly);
    assert!(ui.nodes[node].paint_invalidated_by_hit_test_only);

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(
        paints.load(Ordering::SeqCst),
        1,
        "expected hit-test-only invalidation to replay cached paint when cache key stays stable"
    );
    let stats = ui.debug_stats();
    assert_eq!(
        stats.paint_cache_hit_test_only_replay_allowed, 1,
        "expected hit-test-only gate counter to record replay-allowed attempts"
    );
    assert_eq!(
        stats.paint_cache_hit_test_only_replay_rejected_key_mismatch, 0,
        "expected no key-mismatch rejection when cache key stays stable"
    );
    assert!(!ui.nodes[node].paint_invalidated_by_hit_test_only);
}

#[test]
fn paint_cache_hit_test_only_invalidation_from_descendant_does_not_replay_ancestor() {
    let mut app = crate::test_host::TestHost::new();

    let child_paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_paint_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let child = ui.create_node(CountingPaintWidget {
        paints: child_paints.clone(),
    });
    ui.set_children(root, vec![child]);
    ui.set_root(root);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(child_paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(child, Invalidation::HitTestOnly);
    assert!(ui.nodes[child].paint_invalidated_by_hit_test_only);
    assert!(
        !ui.nodes[root].paint_invalidated_by_hit_test_only,
        "ancestor paint dirtied by descendant hit-test-only invalidation must not replay its cached subtree"
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert!(
        !ui.debug_paint_cache_replays.contains_key(&root),
        "ancestor should not replay cached paint when hit-test-only invalidation came from a descendant"
    );
    assert!(
        ui.debug_paint_cache_replays.contains_key(&child),
        "descendant should still use the local hit-test-only replay path"
    );
    assert_eq!(
        child_paints.load(Ordering::SeqCst),
        1,
        "local descendant hit-test-only invalidation should replay cached paint when the key matches"
    );
}

#[test]
fn paint_cache_hit_test_only_replay_reject_counter_tracks_key_mismatch() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);
    ui.set_debug_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds_a = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds_a, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(node, Invalidation::HitTestOnly);
    assert!(ui.nodes[node].paint_invalidated_by_hit_test_only);

    let bounds_b = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(120.0), fret_core::Px(40.0)),
    );
    ui.paint_all(&mut app, &mut services, bounds_b, &mut scene, 1.0);

    assert_eq!(
        paints.load(Ordering::SeqCst),
        2,
        "expected key mismatch to force repaint even when hit-test-only replay gate is on"
    );
    let stats = ui.debug_stats();
    assert_eq!(
        stats.paint_cache_hit_test_only_replay_allowed, 1,
        "expected hit-test-only gate counter to include key-mismatch attempts"
    );
    assert_eq!(
        stats.paint_cache_hit_test_only_replay_rejected_key_mismatch, 1,
        "expected key-mismatch rejection counter to track rejected replay attempts"
    );
    assert!(!ui.nodes[node].paint_invalidated_by_hit_test_only);
}

#[test]
fn paint_cache_does_not_replay_non_hit_test_invalidations() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.set_root(node);

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 1);

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.invalidate(node, Invalidation::Paint);
    assert!(!ui.nodes[node].paint_invalidated_by_hit_test_only);

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert_eq!(
        paints.load(Ordering::SeqCst),
        2,
        "expected plain paint invalidation to keep forcing repaint"
    );
}

struct ToggleTransformPaintWidget {
    paints: Arc<AtomicUsize>,
    use_transform: Arc<AtomicBool>,
}

impl<H: UiHost> Widget<H> for ToggleTransformPaintWidget {
    fn render_transform(&self, _bounds: Rect) -> Option<Transform2D> {
        self.use_transform
            .load(Ordering::SeqCst)
            .then_some(Transform2D::IDENTITY)
    }

    fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
        self.paints.fetch_add(1, Ordering::SeqCst);
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: cx.bounds,
            background: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
            border: Edges::default(),
            border_paint: fret_core::Paint::Solid(Color::TRANSPARENT).into(),
            corner_radii: Corners::default(),
        });
    }
}
