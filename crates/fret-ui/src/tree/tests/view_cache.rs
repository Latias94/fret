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
        ui.nodes[id].invalidation.clear();
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
    ui.nodes[root].invalidation.layout = false;

    ui.nodes[boundary].bounds = root_bounds;
    ui.nodes[boundary].invalidation.layout = true;

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, root_bounds, 1.0);
    assert!(!ui.nodes[boundary].invalidation.layout);
    assert_eq!(ui.debug_stats().view_cache_contained_relayouts, 1);
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
        ui.nodes[id].invalidation.clear();
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
        ui.nodes[id].invalidation.clear();
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
        ui.nodes[id].invalidation.clear();
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
        ui.nodes[id].invalidation.clear();
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.invalidate_with_source_and_detail(
        leaf,
        Invalidation::HitTestOnly,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::ScrollHandle,
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
        ui.nodes[id].invalidation.clear();
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.invalidate_with_source_and_detail(
        leaf,
        Invalidation::Layout,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::ScrollHandle,
    );

    assert!(ui.nodes[boundary].invalidation.layout);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(
        ui.nodes[boundary].view_cache_needs_rerender,
        "layout-affecting scroll handle changes should force view-cache rerender"
    );
    assert!(!ui.should_reuse_view_cache_node(boundary));
    assert!(!ui.nodes[root].invalidation.paint);
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
        ui.nodes[id].invalidation.clear();
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
        ui.nodes[id].invalidation.clear();
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
