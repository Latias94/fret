use super::*;

#[test]
fn view_cache_invalidation_stops_at_boundary_for_paint() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let a = ui.create_node(TestStack::default());
    let b = ui.create_node(TestStack::default());
    let c = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![a]);
    ui.set_children(a, vec![b]);
    ui.set_children(b, vec![c]);

    for id in [root, a, b, c] {
        ui.test_clear_node_invalidations(id);
    }
    ui.nodes[b].view_cache.enabled = true;
    ui.nodes[b].view_cache.contained_layout = true;

    ui.invalidate(c, Invalidation::Paint);

    assert!(ui.nodes[c].invalidation.paint);
    assert!(ui.nodes[b].invalidation.paint);
    assert!(!ui.nodes[a].invalidation.paint);
    assert!(!ui.nodes[root].invalidation.paint);
    assert_eq!(ui.debug_stats().view_cache_invalidation_truncations, 1);
}

#[test]
fn view_cache_disables_paint_cache_for_non_boundary_nodes() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
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

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);
}

#[test]
fn view_cache_allows_paint_cache_for_boundary_nodes() {
    let mut app = crate::test_host::TestHost::new();

    let paints = Arc::new(AtomicUsize::new(0));
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_paint_cache_enabled(true);

    let node = ui.create_node(CountingPaintWidget {
        paints: paints.clone(),
    });
    ui.nodes[node].view_cache.enabled = true;
    ui.nodes[node].view_cache.contained_layout = true;
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
    assert_eq!(paints.load(Ordering::SeqCst), 1);
}

#[test]
fn view_cache_runs_contained_relayout_for_invalidated_boundaries() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary].view_cache.contained_layout = true;

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);

    let root_bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.nodes[root].bounds = root_bounds;
    ui.nodes[root].measured_size = root_bounds.size;
    ui.test_set_layout_invalidation(root, false);

    ui.nodes[boundary].bounds = root_bounds;
    ui.test_set_layout_invalidation(boundary, true);

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, root_bounds, 1.0);
    assert!(!ui.nodes[boundary].invalidation.layout);
    assert_eq!(ui.debug_stats().view_cache_contained_relayouts, 1);
}

#[test]
fn view_cache_mark_nearest_root_needs_rerender_propagates_to_ancestor_roots() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack::default());
    let outer = ui.create_node(TestStack::default());
    let mid = ui.create_node(TestStack::default());
    let inner = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [root, outer, mid, inner, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }
    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[inner].view_cache.enabled = true;

    ui.mark_nearest_view_cache_root_needs_rerender(
        leaf,
        UiDebugInvalidationSource::Notify,
        UiDebugInvalidationDetail::ScrollHandleLayout,
    );

    assert!(
        ui.nodes[inner].view_cache_needs_rerender,
        "expected nearest cache root to be marked for rerender"
    );
    assert!(
        ui.nodes[outer].view_cache_needs_rerender,
        "expected ancestor cache roots to be marked for rerender"
    );

    // Ensure the dirty-view list is surfaced in debug snapshots.
    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let dirty = ui.debug_dirty_views();
    assert!(
        dirty
            .iter()
            .any(|d| d.view.0 == inner && d.detail == UiDebugInvalidationDetail::ScrollHandleLayout),
        "expected dirty views to include inner cache root with ScrollHandleLayout detail"
    );
    assert!(
        dirty
            .iter()
            .any(|d| d.view.0 == outer && d.detail == UiDebugInvalidationDetail::ScrollHandleLayout),
        "expected dirty views to include outer cache root with ScrollHandleLayout detail"
    );
}

#[test]
fn view_cache_auto_sized_repair_does_not_promote_hit_test_when_bounds_are_known() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary].view_cache.contained_layout = true;
    ui.nodes[boundary].view_cache.layout_definite = false;

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);

    for id in [root, boundary] {
        ui.test_clear_node_invalidations(id);
    }

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.nodes[boundary].bounds = bounds;
    ui.nodes[boundary].measured_size = bounds.size;

    ui.invalidate(boundary, Invalidation::HitTestOnly);
    assert!(!ui.nodes[boundary].invalidation.layout);
    assert!(ui.nodes[boundary].invalidation.hit_test);

    ui.propagate_auto_sized_view_cache_root_invalidations();

    assert!(!ui.nodes[boundary].invalidation.layout);
    assert!(!ui.nodes[root].invalidation.layout);
}

#[test]
fn view_cache_nested_boundaries_invalidate_ancestor_cache_roots() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let outer = ui.create_node(TestStack::default());
    let mid = ui.create_node(TestStack::default());
    let inner = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [root, outer, mid, inner, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer].view_cache.contained_layout = true;
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner].view_cache.contained_layout = true;

    ui.invalidate(leaf, Invalidation::Paint);

    assert!(ui.nodes[leaf].invalidation.paint);
    assert!(ui.nodes[inner].invalidation.paint);
    assert!(ui.nodes[outer].invalidation.paint);
    assert!(!ui.nodes[mid].invalidation.paint);
    assert!(!ui.nodes[root].invalidation.paint);
}

#[test]
fn view_cache_notify_marks_cache_root_needs_rerender() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary].view_cache.contained_layout = true;

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.mark_invalidation_with_source(leaf, Invalidation::Paint, UiDebugInvalidationSource::Notify);

    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(ui.nodes[boundary].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_notify_propagates_to_ancestor_cache_roots() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let outer = ui.create_node(TestStack::default());
    let mid = ui.create_node(TestStack::default());
    let inner = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer].view_cache.contained_layout = true;
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner].view_cache.contained_layout = true;

    for id in [root, outer, mid, inner, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.mark_invalidation_with_source(leaf, Invalidation::Paint, UiDebugInvalidationSource::Notify);

    assert!(ui.nodes[inner].view_cache_needs_rerender);
    assert!(ui.nodes[outer].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(inner));
    assert!(!ui.should_reuse_view_cache_node(outer));
}

#[test]
fn view_cache_scroll_handle_hit_test_only_invalidations_do_not_mark_cache_root_needs_rerender() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.nodes[boundary].bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
    );

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.invalidate_with_source_and_detail(
        leaf,
        Invalidation::HitTestOnly,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::ScrollHandleHitTestOnly,
    );

    assert!(ui.nodes[boundary].invalidation.hit_test);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "scroll-handle hit-test-only invalidations should not force view-cache rerender"
    );
    assert!(
        ui.should_reuse_view_cache_node(boundary),
        "hit-test-only invalidations should allow view-cache reuse"
    );
    assert!(!ui.nodes[root].invalidation.paint);
}

#[test]
fn view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let leaf = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.nodes[boundary].bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
    );

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.invalidate_with_source_and_detail(
        leaf,
        Invalidation::Layout,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::ScrollHandleLayout,
    );

    assert!(ui.nodes[boundary].invalidation.layout);
    assert!(ui.nodes[boundary].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(boundary));
    assert!(!ui.nodes[root].invalidation.paint);
}

#[test]
fn view_cache_scroll_handle_window_update_marks_cache_root_needs_rerender() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let vlist_node = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![vlist_node]);

    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.nodes[boundary].bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let scroll_handle = crate::scroll::VirtualListScrollHandle::new();
    let handle_key = scroll_handle.base_handle().binding_key();

    // Seed element state with a previously rendered overscan window.
    let vlist_element = crate::GlobalElementId(1);
    let len = 100usize;
    let overscan = 2usize;
    let viewport = fret_core::Px(100.0);
    let mut metrics = crate::virtual_list::VirtualListMetrics::default();
    metrics.ensure_with_mode(
        crate::element::VirtualListMeasureMode::Fixed,
        len,
        fret_core::Px(10.0),
        fret_core::Px(0.0),
        fret_core::Px(0.0),
    );
    let initial_window = metrics
        .visible_range(fret_core::Px(0.0), viewport, overscan)
        .expect("initial window range");

    crate::elements::with_element_state(
        &mut app,
        window,
        vlist_element,
        crate::element::VirtualListState::default,
        |state| {
            state.viewport_h = viewport;
            state.metrics = metrics.clone();
            state.render_window_range = Some(initial_window);
        },
    );

    // Register the element instance + scroll-handle binding used by the invalidation pass.
    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        window_frame.instances.insert(
            vlist_node,
            crate::declarative::frame::ElementRecord {
                element: vlist_element,
                instance: crate::declarative::frame::ElementInstance::VirtualList(
                    crate::element::VirtualListProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: fret_core::Axis::Vertical,
                        len,
                        items_revision: 0,
                        estimate_row_height: fret_core::Px(10.0),
                        measure_mode: crate::element::VirtualListMeasureMode::Fixed,
                        key_cache: crate::element::VirtualListKeyCacheMode::AllKeys,
                        overscan,
                        keep_alive: 0,
                        scroll_margin: fret_core::Px(0.0),
                        gap: fret_core::Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
            },
        );
    });

    let frame_id = app.frame_id();
    crate::declarative::frame::register_scroll_handle_bindings_batch(
        &mut app,
        window,
        frame_id,
        [crate::declarative::frame::ScrollHandleBinding {
            handle_key,
            element: vlist_element,
            handle: scroll_handle.base_handle().clone(),
        }],
    );

    // Prime scroll-handle revisions so the next change is treated as a delta.
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
    );
    for id in [root, boundary, vlist_node] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    // Scroll far enough to fall outside the previously rendered overscan window.
    scroll_handle.set_offset(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
    );

    assert!(ui.nodes[boundary].invalidation.hit_test);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(ui.nodes[boundary].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_scroll_windowed_paint_marks_cache_root_needs_rerender() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let scroll_node = ui.create_node(TestStack::default());

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![scroll_node]);

    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.nodes[boundary].bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );

    let scroll_handle = crate::scroll::ScrollHandle::default();
    let handle_key = scroll_handle.binding_key();

    // Register the element instance + scroll-handle binding used by the invalidation pass.
    let scroll_element = crate::GlobalElementId(2);
    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        window_frame.instances.insert(
            scroll_node,
            crate::declarative::frame::ElementRecord {
                element: scroll_element,
                instance: crate::declarative::frame::ElementInstance::Scroll(
                    crate::element::ScrollProps {
                        layout: crate::element::LayoutStyle::default(),
                        axis: crate::element::ScrollAxis::Y,
                        scroll_handle: Some(scroll_handle.clone()),
                        intrinsic_measure_mode: crate::element::ScrollIntrinsicMeasureMode::Content,
                        windowed_paint: true,
                        probe_unbounded: true,
                    },
                ),
            },
        );
    });

    let frame_id = app.frame_id();
    crate::declarative::frame::register_scroll_handle_bindings_batch(
        &mut app,
        window,
        frame_id,
        [crate::declarative::frame::ScrollHandleBinding {
            handle_key,
            element: scroll_element,
            handle: scroll_handle.clone(),
        }],
    );

    // Prime scroll-handle revisions so the next change is treated as a delta.
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
    );
    for id in [root, boundary, scroll_node] {
        ui.nodes[id].invalidation.clear();
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    // Programmatic scroll should mark the cache root dirty so windowed paint surfaces can update.
    scroll_handle.set_offset(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
    );

    assert!(ui.nodes[boundary].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(boundary));
    // The scroll node itself remains hit-test-only invalidated; the rerender flag carries the
    // windowed-paint contract.
    assert!(ui.nodes[scroll_node].invalidation.hit_test);
}

#[test]
fn widget_request_animation_frame_marks_nearest_view_cache_root_dirty() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    struct RafWidget;

    impl<H: UiHost> Widget<H> for RafWidget {
        fn paint(&mut self, cx: &mut PaintCx<'_, H>) {
            cx.request_animation_frame();
        }
    }

    let root = ui.create_node(TestStack::default());
    let boundary = ui.create_node(TestStack::default());
    let leaf = ui.create_node(RafWidget);

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary].view_cache.contained_layout = true;

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    let mut services = FakeUiServices;
    let mut scene = Scene::default();
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        ui.nodes[boundary].view_cache_needs_rerender,
        "request_animation_frame should behave like notify(view) and disable view-cache reuse"
    );
    assert!(!ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack::default());
    let outer = ui.create_node(TestStack::default());
    let inner = ui.create_node(TestStack::default());
    let leaf = ui.create_node(PaintObservingWidget {
        model: model.clone(),
    });

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer].view_cache.contained_layout = true;
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner].view_cache.contained_layout = true;

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    for id in [root, outer, inner, leaf] {
        ui.test_clear_node_invalidations(id);
    }

    let observed = ui
        .observed_in_paint
        .by_model
        .get(&model.id())
        .expect("expected paint observation for model");
    assert!(
        observed.contains_key(&inner),
        "nearest cache root should observe"
    );
    assert!(
        !observed.contains_key(&leaf),
        "leaf observation should be uplifted to cache root in view-cache mode"
    );
    assert!(
        !observed.contains_key(&outer),
        "observation should not be attributed to ancestor cache roots"
    );

    let _ = model.update(&mut app, |v, _cx| *v += 1);
    let changed = app.take_changed_models();
    ui.propagate_model_changes(&mut app, &changed);

    assert!(ui.nodes[inner].invalidation.paint);
    assert!(ui.nodes[outer].invalidation.paint);
    assert!(
        ui.nodes[inner].view_cache_needs_rerender,
        "model change should mark nearest cache root as dirty"
    );
    assert!(
        ui.nodes[outer].view_cache_needs_rerender,
        "nested cache-root correctness requires dirty propagation to ancestor cache roots"
    );
    assert!(!ui.nodes[root].invalidation.paint);
}
