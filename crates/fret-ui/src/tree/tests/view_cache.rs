use super::*;
use std::any::TypeId;

#[test]
fn view_cache_invalidation_stops_at_boundary_for_paint() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let a = ui.create_node(TestStack);
    let b = ui.create_node(TestStack);
    let c = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![a]);
    ui.set_children(a, vec![b]);
    ui.set_children(b, vec![c]);

    for id in [root, a, b, c] {
        ui.test_clear_node_invalidations(id);
    }
    ui.nodes[b].view_cache.enabled = true;
    ui.nodes[b]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

    ui.invalidate(c, Invalidation::Paint);

    assert!(ui.nodes[c].invalidation.paint);
    assert!(ui.nodes[b].invalidation.paint);
    assert!(!ui.nodes[a].invalidation.paint);
    assert!(!ui.nodes[root].invalidation.paint);
    assert_eq!(ui.debug_stats().view_cache_invalidation_truncations, 1);
}

#[test]
fn view_cache_invalidation_walk_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

    ui.test_set_node_parent(leaf, None);

    ui.invalidate(leaf, Invalidation::Paint);

    assert!(ui.nodes[leaf].invalidation.paint);
    assert!(
        ui.nodes[boundary].invalidation.paint,
        "invalidation must walk to the actual child-edge cache boundary"
    );
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
    assert!(
        !ui.test_paint_cache_entry_for_node_has_entry(node),
        "view-cache-active non-cache nodes should not record paint-cache entries"
    );

    ui.ingest_paint_cache_source(&mut scene);
    scene.clear();

    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    assert_eq!(paints.load(Ordering::SeqCst), 2);
    assert!(
        !ui.test_paint_cache_entry_for_node_has_entry(node),
        "view-cache-active non-cache nodes should keep paint-cache disabled"
    );
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
        "view-cache boundary nodes should store replay entries in ViewBoundaryState"
    );
    assert!(
        !ui.test_retained_paint_cache_entry_store_has_entry(node),
        "view-cache boundary nodes should not use the retained plain-node paint-cache entry store"
    );

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

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

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
    assert_eq!(ui.nodes[boundary].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);
    let stats = ui.debug_stats();
    assert_eq!(stats.view_cache_contained_relayouts, 1);
    assert_eq!(stats.dirty_frontier_boundaries_max, 1);
    assert_eq!(stats.dirty_frontier_boundaries_at_layout_start, 1);
    assert_eq!(stats.dirty_frontier_contained_candidates, 1);
}

#[test]
fn descendant_layout_invalidation_marks_contained_view_cache_root_dirty() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[boundary].view_cache.layout_definite = true;

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.nodes[root].bounds = bounds;
    ui.nodes[root].measured_size = bounds.size;
    ui.nodes[boundary].bounds = bounds;
    ui.nodes[boundary].measured_size = bounds.size;
    ui.nodes[leaf].bounds = bounds;
    ui.nodes[leaf].measured_size = bounds.size;

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [root, boundary, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    assert!(
        ui.test_dirty_view_frontier_empty(),
        "stable frames must not carry contained-relayout boundary candidates from the initial mount"
    );

    ui.invalidate(leaf, Invalidation::Layout);

    assert!(ui.nodes[leaf].invalidation.layout);
    assert!(ui.nodes[boundary].invalidation.layout);
    assert!(
        ui.test_view_boundary_layout_dirty(boundary),
        "contained boundaries with descendant layout invalidations must remain discoverable for the contained relayout pass"
    );
    let dirty_boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|stats| stats.id == boundary)
        .expect("expected contained cache root boundary stats");
    assert!(dirty_boundary.layout_dirty);
    assert_eq!(
        dirty_boundary.layout_dirty_source,
        Some(UiDebugInvalidationSource::Other)
    );
    assert_eq!(
        dirty_boundary.layout_dirty_detail,
        Some(UiDebugInvalidationDetail::SubtreeLayoutDirtyRepair)
    );
    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "layout-only descendant invalidations should schedule contained relayout without escalating to declarative rerender"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.debug_stats().view_cache_contained_relayouts, 1);
    assert!(
        !ui.nodes[boundary].invalidation.layout,
        "contained relayout must consume the cache-root layout invalidation induced by the descendant"
    );
    assert!(
        !ui.nodes[leaf].invalidation.layout,
        "contained relayout must converge the descendant layout invalidation in the same frame"
    );
}

#[test]
fn view_cache_contained_relayout_does_not_force_next_frame_rerender() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[boundary].view_cache.layout_definite = true;

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
    ui.nodes[boundary].measured_size = root_bounds.size;
    ui.nodes[boundary].view_cache_needs_rerender = false;
    ui.test_set_layout_invalidation(boundary, true);

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, root_bounds, 1.0);

    assert!(
        !ui.nodes[boundary].invalidation.layout,
        "contained relayout should clear the layout invalidation it just consumed"
    );
    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "layout-only contained relayout must not escalate into next-frame declarative rerender"
    );
    assert!(
        ui.should_reuse_view_cache_node(boundary),
        "once contained relayout clears layout invalidation, the cache root should remain reusable"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, root_bounds, 1.0);

    assert!(
        ui.debug_dirty_views().is_empty(),
        "clean contained-relayout roots should not remain in dirty-view diagnostics on the next frame"
    );
    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "the cache root should stay clean across the following stable frame"
    );
}

#[test]
fn layout_in_skips_clean_root_even_when_another_node_is_layout_dirty() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let clean_layouts = Arc::new(AtomicUsize::new(0));
    let clean_root = ui.create_node(CountingLayoutWidget {
        layouts: Arc::clone(&clean_layouts),
    });
    let dirty_node = ui.create_node(TestStack);

    ui.set_root(clean_root);
    ui.set_children(clean_root, Vec::new());

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(clean_layouts.load(Ordering::SeqCst), 1);

    ui.test_clear_node_invalidations(clean_root);
    ui.nodes[clean_root].bounds = bounds;
    ui.nodes[clean_root].measured_size = bounds.size;

    ui.invalidate(dirty_node, Invalidation::Layout);
    let layout_visited_before = ui.debug_stats().layout_nodes_visited;
    let size = ui.layout_in(&mut app, &mut services, clean_root, bounds, 1.0);

    assert_eq!(size, bounds.size);
    assert_eq!(
        clean_layouts.load(Ordering::SeqCst),
        1,
        "expected the clean root to reuse its cached layout even when another node is dirty"
    );
    assert_eq!(
        ui.debug_stats().layout_nodes_visited,
        layout_visited_before,
        "expected the clean root to skip entering the layout engine"
    );

    ui.test_clear_node_invalidations(dirty_node);
    ui.invalidate(dirty_node, Invalidation::HitTestOnly);
    let hit_test_visited_before = ui.debug_stats().layout_nodes_visited;
    let size = ui.layout_in(&mut app, &mut services, clean_root, bounds, 1.0);

    assert_eq!(size, bounds.size);
    assert_eq!(
        clean_layouts.load(Ordering::SeqCst),
        1,
        "expected the clean root to reuse its cached layout even when another node only needs hit-test repair"
    );
    assert_eq!(
        ui.debug_stats().layout_nodes_visited,
        hit_test_visited_before,
        "expected a detached hit-test-only invalidation to stay out of the clean root fast path"
    );
}

#[test]
fn layout_in_skips_hit_test_only_root_when_geometry_is_clean() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let layouts = Arc::new(AtomicUsize::new(0));
    let root = ui.create_node(CountingLayoutWidget {
        layouts: Arc::clone(&layouts),
    });
    ui.set_root(root);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(layouts.load(Ordering::SeqCst), 1);

    ui.test_clear_node_invalidations(root);
    ui.invalidate(root, Invalidation::HitTestOnly);

    let visited_before = ui.debug_stats().layout_nodes_visited;
    let size = ui.layout_in(&mut app, &mut services, root, bounds, 1.0);

    assert_eq!(size, bounds.size);
    assert_eq!(
        layouts.load(Ordering::SeqCst),
        1,
        "hit-test-only dirty state should not force the root widget layout"
    );
    assert_eq!(
        ui.debug_stats().layout_nodes_visited,
        visited_before,
        "hit-test-only dirty state should stay on the root layout fast path"
    );
}

#[test]
fn view_cache_layout_dirty_expansion_reaches_clean_nested_cache_root_descendants() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [outer, inner] {
        ui.set_node_view_cache_flags(id, true, true, true);
        ui.nodes[id].bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
        );
        ui.nodes[id].measured_size = ui.nodes[id].bounds.size;
    }

    for id in [root, outer, inner, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.test_set_layout_invalidation(outer, true);

    ui.expand_view_cache_layout_invalidations_if_needed();

    assert!(ui.nodes[outer].invalidation.layout);
    assert!(ui.nodes[inner].invalidation.layout);
    assert!(
        ui.nodes[leaf].invalidation.layout,
        "layout dirty expansion must pass through clean nested cache roots so cached descendant geometry can be refreshed"
    );
}

#[test]
fn view_cache_layout_dirty_expansion_keeps_dirty_nested_cache_root() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [outer, inner] {
        ui.set_node_view_cache_flags(id, true, true, true);
        ui.nodes[id].bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
        );
        ui.nodes[id].measured_size = ui.nodes[id].bounds.size;
    }

    for id in [root, outer, inner, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.test_set_layout_invalidation(inner, true);
    ui.test_set_layout_invalidation(outer, true);

    ui.expand_view_cache_layout_invalidations_if_needed();

    assert!(ui.nodes[outer].invalidation.layout);
    assert!(ui.nodes[inner].invalidation.layout);
    assert!(
        ui.nodes[leaf].invalidation.layout,
        "dirty nested cache roots must still expand into descendants"
    );
}

#[test]
fn view_cache_layout_dirty_expansion_does_not_prune_non_contained_nested_cache_root() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    ui.set_node_view_cache_flags(outer, true, true, true);
    ui.set_node_view_cache_flags(inner, true, false, true);

    for id in [root, outer, inner, leaf] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.test_set_layout_invalidation(outer, true);
    ui.expand_view_cache_layout_invalidations_if_needed();

    assert!(ui.nodes[outer].invalidation.layout);
    assert!(ui.nodes[inner].invalidation.layout);
    assert!(
        ui.nodes[leaf].invalidation.layout,
        "non-contained nested cache roots should still receive layout dirtiness"
    );
}

#[test]
fn detached_dirty_view_cache_root_is_pruned_before_layout_followups() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[boundary].view_cache.layout_definite = true;

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(100.0), fret_core::Px(40.0)),
    );
    ui.nodes[root].bounds = bounds;
    ui.nodes[root].measured_size = bounds.size;
    ui.nodes[boundary].bounds = bounds;
    ui.nodes[boundary].measured_size = bounds.size;

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(boundary);
    ui.test_set_layout_invalidation(boundary, true);
    let boundary_view = ui
        .test_view_id_for_boundary_node(boundary)
        .expect("dirty boundary view id");

    ui.set_children(root, Vec::new());
    assert_eq!(ui.debug_node_parent_storage(boundary), None);

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !ui.debug_view_cache_contained_relayout_roots()
            .contains(&boundary),
        "detached cache roots must not be scheduled for contained relayout"
    );
    assert!(
        ui.debug_dirty_views()
            .iter()
            .all(|dirty| dirty.view != boundary_view),
        "detached cache roots must not survive in dirty-view diagnostics"
    );

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ui.debug_stats().layout_skipped_engine_frame || ui.debug_stats().layout_fast_path_taken,
        "detached dirty cache roots must not block the next stable frame from taking a layout-skip path"
    );
}

#[test]
fn view_cache_mark_nearest_root_needs_rerender_propagates_to_ancestor_roots() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let mid = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

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
    let inner_view = ui
        .test_view_id_for_boundary_node(inner)
        .expect("inner boundary view id");
    let outer_view = ui
        .test_view_id_for_boundary_node(outer)
        .expect("outer boundary view id");

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
            .any(|d| d.view == inner_view
                && d.detail == UiDebugInvalidationDetail::ScrollHandleLayout),
        "expected dirty views to include inner cache root with ScrollHandleLayout detail"
    );
    assert!(
        dirty
            .iter()
            .any(|d| d.view == outer_view
                && d.detail == UiDebugInvalidationDetail::ScrollHandleLayout),
        "expected dirty views to include outer cache root with ScrollHandleLayout detail"
    );
}

#[test]
fn view_cache_nearest_root_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let actual_outer = ui.create_node(TestStack);
    let stale_outer = ui.create_node(TestStack);
    let actual_inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![actual_outer, stale_outer]);
    ui.set_children(actual_outer, vec![actual_inner]);
    ui.set_children(actual_inner, vec![leaf]);

    for id in [actual_outer, stale_outer, actual_inner] {
        ui.nodes[id].view_cache.enabled = true;
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.test_set_node_parent(leaf, Some(stale_outer));

    ui.mark_nearest_view_cache_root_needs_rerender(
        leaf,
        UiDebugInvalidationSource::Notify,
        UiDebugInvalidationDetail::ScrollHandleLayout,
    );

    assert!(
        ui.nodes[actual_inner].view_cache_needs_rerender,
        "nearest cache root must follow child-edge topology, not the stale retained parent"
    );
    assert!(
        ui.nodes[actual_outer].view_cache_needs_rerender,
        "ancestor cache roots must follow child-edge topology"
    );
    assert!(
        !ui.nodes[stale_outer].view_cache_needs_rerender,
        "stale retained parents must not receive cache-root rerender pressure"
    );
}

#[test]
fn view_boundary_frame_products_track_live_topology_epoch_after_reparent() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let left = ui.create_node(TestStack);
    let right = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![left, right]);
    ui.set_children(left, vec![boundary]);
    ui.set_node_view_cache_flags(left, true, true, true);
    ui.set_node_view_cache_flags(right, true, true, true);
    ui.set_node_view_cache_flags(boundary, true, true, true);

    let first_epoch = ui.live_topology_epoch();
    assert_eq!(ui.test_view_boundary_parent(boundary), Some(left));
    assert_eq!(
        ui.test_view_boundary_topology_epoch(boundary),
        Some(first_epoch)
    );

    ui.test_set_node_parent(boundary, None);
    ui.set_children(right, vec![boundary]);

    let next_epoch = ui.live_topology_epoch();
    assert!(next_epoch > first_epoch);
    assert_eq!(ui.test_view_boundary_parent(boundary), Some(right));
    assert_eq!(
        ui.test_view_boundary_topology_epoch(boundary),
        Some(next_epoch)
    );

    let stats = ui.debug_boundary_stats();
    let boundary_stats = stats
        .iter()
        .find(|stats| stats.id == boundary)
        .expect("boundary stats for reparented node");
    assert_eq!(boundary_stats.parent, Some(right));
    assert_eq!(boundary_stats.topology_epoch, next_epoch.as_u64());
}

#[test]
fn contained_view_cache_dirty_coverage_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let wrapper = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![wrapper]);
    ui.set_children(wrapper, vec![boundary]);

    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[boundary].invalidation.layout = true;
    ui.nodes[boundary].subtree_layout_dirty_count = 1;
    ui.nodes[wrapper].invalidation.layout = false;
    ui.nodes[wrapper].subtree_layout_dirty_count = 1;
    ui.mark_boundary_layout_dirty(
        boundary,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::SubtreeLayoutDirtyRepair,
    );

    ui.test_set_node_parent(boundary, None);

    assert!(
        ui.node_subtree_layout_dirty_covered_by_contained_view_cache_roots(wrapper),
        "contained dirty coverage must follow child-edge topology, not retained parent pointers"
    );
}

#[test]
fn contained_view_cache_relayout_uses_child_edges_for_candidate_pruning_and_scroll_followup() {
    struct SpyScroll {
        layout_count: Arc<AtomicUsize>,
    }

    impl<H: UiHost> Widget<H> for SpyScroll {
        fn can_scroll_descendant_into_view(&self) -> bool {
            true
        }

        fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
            self.layout_count.fetch_add(1, Ordering::SeqCst);
            for &child in cx.children {
                let _ = cx.layout_in(child, cx.bounds);
            }
            cx.available
        }

        fn paint(&mut self, _cx: &mut PaintCx<'_, H>) {}
    }

    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let actual_scroll_layouts = Arc::new(AtomicUsize::new(0));
    let stale_scroll_layouts = Arc::new(AtomicUsize::new(0));

    let root = ui.create_node(TestStack);
    let actual_scroll = ui.create_node(SpyScroll {
        layout_count: actual_scroll_layouts.clone(),
    });
    let stale_scroll = ui.create_node(SpyScroll {
        layout_count: stale_scroll_layouts.clone(),
    });
    let outer = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![actual_scroll, stale_scroll]);
    ui.set_children(actual_scroll, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [outer, inner] {
        ui.set_node_view_cache_flags(id, true, true, true);
        ui.nodes[id].view_cache.layout_definite = true;
    }

    let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(100.0), Px(40.0)));
    ui.nodes[root].bounds = bounds;
    ui.nodes[root].measured_size = bounds.size;
    for id in [actual_scroll, stale_scroll, outer, inner, leaf] {
        ui.nodes[id].bounds = bounds;
        ui.nodes[id].measured_size = bounds.size;
    }

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [root, actual_scroll, stale_scroll, outer, inner, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    ui.test_set_node_parent(inner, Some(stale_scroll));
    actual_scroll_layouts.store(0, Ordering::SeqCst);
    stale_scroll_layouts.store(0, Ordering::SeqCst);

    ui.test_set_layout_invalidation(outer, true);
    ui.test_set_layout_invalidation(inner, true);
    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.debug_view_cache_contained_relayout_roots(),
        &[outer],
        "contained relayout candidate pruning must follow child-edge ancestors"
    );
    assert!(
        actual_scroll_layouts.load(Ordering::SeqCst) > 0,
        "scroll follow-up must target the actual child-edge scroll ancestor"
    );
    assert_eq!(
        stale_scroll_layouts.load(Ordering::SeqCst),
        0,
        "stale retained parents must not receive scroll follow-up relayouts"
    );
}

#[test]
fn view_cache_auto_sized_repair_does_not_promote_hit_test_when_bounds_are_known() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
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

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let mid = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [root, outer, mid, inner, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

    ui.invalidate(leaf, Invalidation::Paint);

    assert!(ui.nodes[leaf].invalidation.paint);
    assert!(ui.nodes[inner].invalidation.paint);
    assert!(ui.nodes[outer].invalidation.paint);
    assert!(!ui.nodes[mid].invalidation.paint);
    assert!(!ui.nodes[root].invalidation.paint);
}

#[test]
fn view_cache_nested_boundary_ancestors_use_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let stale_outer = ui.create_node(TestStack);
    let mid = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer, stale_outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    for id in [root, outer, stale_outer, mid, inner, leaf] {
        ui.test_clear_node_invalidations(id);
    }
    for id in [outer, stale_outer, inner] {
        ui.nodes[id].view_cache.enabled = true;
        ui.nodes[id]
            .view_cache
            .test_set_layout_contained_when_bounds_known(true);
    }

    ui.test_set_node_parent(inner, Some(stale_outer));

    ui.invalidate(leaf, Invalidation::Paint);

    assert!(ui.nodes[inner].invalidation.paint);
    assert!(
        ui.nodes[outer].invalidation.paint,
        "nested cache-root propagation must mark the actual child-edge ancestor"
    );
    assert!(
        !ui.nodes[stale_outer].invalidation.paint,
        "stale retained parents must not receive nested cache-root invalidation"
    );
}

#[test]
fn view_cache_notify_marks_cache_root_needs_rerender() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

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

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let mid = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![mid]);
    ui.set_children(mid, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

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

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

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
fn view_cache_layout_invalidations_allow_reuse_for_definite_contained_roots() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);

    ui.set_node_view_cache_flags(boundary, true, true, true);
    ui.nodes[boundary].bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(10.0), fret_core::Px(10.0)),
    );
    ui.nodes[boundary].measured_size = ui.nodes[boundary].bounds.size;

    for id in [root, boundary] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    ui.test_set_layout_invalidation(boundary, true);

    assert!(ui.nodes[boundary].invalidation.layout);
    assert!(
        ui.should_reuse_view_cache_node(boundary),
        "layout invalidations should not disable view-cache reuse for definite contained roots"
    );
}

#[test]
fn view_cache_scroll_handle_layout_invalidations_mark_cache_root_needs_rerender() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

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

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let vlist_node = ui.create_node(TestStack);

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
                        effective_overscan: overscan,
                        keep_alive: 0,
                        scroll_margin: fret_core::Px(0.0),
                        gap: fret_core::Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
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
        false,
        true,
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
        false,
        true,
    );

    assert!(ui.nodes[boundary].invalidation.hit_test);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(ui.nodes[boundary].view_cache_needs_rerender);
    assert!(!ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_scroll_windowed_paint_marks_cache_root_paint_dirty_without_rerender() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let scroll_node = ui.create_node(TestStack);

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
                        known_content_size: None,
                        intrinsic_measure_mode: crate::element::ScrollIntrinsicMeasureMode::Content,
                        windowed_paint: true,
                        probe_unbounded: true,
                    },
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
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
        false,
        true,
    );
    for id in [root, boundary, scroll_node] {
        ui.nodes[id].invalidation.clear();
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    // Programmatic scroll should repaint the cache root so windowed paint surfaces can update
    // without rerunning the declarative cache-root closure.
    scroll_handle.set_offset(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "windowed-paint offset changes should repaint without forcing a view-cache rerender"
    );
    assert!(ui.should_reuse_view_cache_node(boundary));
    assert!(
        ui.nodes[boundary].invalidation.paint,
        "windowed-paint offset changes must still invalidate paint-cache replay"
    );
    // The scroll node itself remains hit-test-only invalidated; the cache root paint invalidation
    // carries the windowed-paint contract.
    assert!(ui.nodes[scroll_node].invalidation.hit_test);
}

#[test]
fn view_cache_scroll_windowed_paint_revision_only_bump_after_internal_offset_update_stays_hit_test_only()
 {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let scroll_node = ui.create_node(TestStack);

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

    let scroll_element = crate::GlobalElementId(3);
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
                        known_content_size: None,
                        intrinsic_measure_mode: crate::element::ScrollIntrinsicMeasureMode::Content,
                        windowed_paint: true,
                        probe_unbounded: true,
                    },
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
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

    // Prime the registry so later checks observe deltas from the authoritative baselines.
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );
    for id in [root, boundary, scroll_node] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    // Simulate a runtime-driven offset sync that should update baselines without producing a
    // revision delta.
    scroll_handle.set_offset_internal(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    for id in [root, boundary, scroll_node] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    let prev_change_count = ui.debug_scroll_handle_changes().len();
    let prev_walk_count = ui.debug_invalidation_walks().len();

    // The next revision-only bump must see the updated baseline and stay off the window-update
    // path for windowed paint.
    scroll_handle.bump_revision();
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    let changes = &ui.debug_scroll_handle_changes()[prev_change_count..];
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].handle_key, handle_key);
    assert_eq!(
        changes[0].kind,
        crate::tree::UiDebugScrollHandleChangeKind::Layout
    );
    assert!(
        !changes[0].offset_changed && !changes[0].viewport_changed && !changes[0].content_changed,
        "revision-only bump must reuse the internal-update baseline instead of reclassifying as an offset change"
    );

    let walks = &ui.debug_invalidation_walks()[prev_walk_count..];
    assert!(
        walks.iter().any(|walk| {
            walk.inv == Invalidation::HitTestOnly
                && walk.detail == UiDebugInvalidationDetail::ScrollHandleHitTestOnly
        }),
        "final invalidation should downgrade revision-only bumps to hit-test-only"
    );
    assert!(
        walks.iter().all(|walk| {
            !(walk.inv == Invalidation::Layout
                && walk.detail == UiDebugInvalidationDetail::ScrollHandleLayout)
        }),
        "revision-only bumps must not force a layout invalidation walk for windowed paint"
    );

    assert!(ui.nodes[boundary].invalidation.hit_test);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(
        !ui.nodes[boundary].view_cache_needs_rerender,
        "windowed-paint cache roots should stay reusable when the follow-up bump is revision-only"
    );
    assert!(ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_virtual_list_revision_only_bump_after_internal_offset_update_marks_window_update() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let vlist_node = ui.create_node(TestStack);

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

    let vlist_element = crate::GlobalElementId(4);
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
                        effective_overscan: overscan,
                        keep_alive: 0,
                        scroll_margin: fret_core::Px(0.0),
                        gap: fret_core::Px(0.0),
                        scroll_handle: scroll_handle.clone(),
                        visible_items: Vec::new(),
                    },
                ),
                inherited_foreground: None,
                inherited_text_style: None,
                semantics_decoration: None,
                key_context: None,
                layout_direction: fret_core::LayoutDirection::default(),
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

    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );
    for id in [root, boundary, vlist_node] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    scroll_handle.set_offset_internal(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    for id in [root, boundary, vlist_node] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    let prev_change_count = ui.debug_scroll_handle_changes().len();
    let prev_walk_count = ui.debug_invalidation_walks().len();

    scroll_handle.bump_revision();
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    let changes = &ui.debug_scroll_handle_changes()[prev_change_count..];
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].handle_key, handle_key);
    assert_eq!(
        changes[0].kind,
        crate::tree::UiDebugScrollHandleChangeKind::Layout
    );
    assert!(
        !changes[0].offset_changed && !changes[0].viewport_changed && !changes[0].content_changed,
        "revision-only bump must see the post-layout internal offset as the last committed baseline"
    );

    let walks = &ui.debug_invalidation_walks()[prev_walk_count..];
    assert!(
        walks.iter().any(|walk| {
            walk.inv == Invalidation::HitTestOnly
                && walk.detail == UiDebugInvalidationDetail::ScrollHandleHitTestOnly
        }),
        "final invalidation should still downgrade revision-only bumps to hit-test-only before window-update escalation"
    );
    assert!(
        walks.iter().all(|walk| {
            !(walk.inv == Invalidation::Layout
                && walk.detail == UiDebugInvalidationDetail::ScrollHandleLayout)
        }),
        "window mismatch should not force a layout invalidation walk when the change is revision-only"
    );

    assert!(ui.nodes[boundary].invalidation.hit_test);
    assert!(ui.nodes[boundary].invalidation.paint);
    assert!(
        ui.nodes[boundary].view_cache_needs_rerender,
        "revision-only bumps must still trigger a window update when the visible range escaped the cached overscan window"
    );
    assert!(!ui.should_reuse_view_cache_node(boundary));
}

#[test]
fn view_cache_scroll_handle_ignores_detached_same_frame_stale_bindings() {
    let mut app = crate::test_host::TestHost::new();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let live_boundary = ui.create_node(TestStack);
    let live_scroll = ui.create_node(TestStack);
    let stale_boundary = ui.create_node(TestStack);
    let stale_scroll = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![live_boundary]);
    ui.set_children(live_boundary, vec![live_scroll]);
    ui.set_children(stale_boundary, vec![stale_scroll]);

    for boundary in [live_boundary, stale_boundary] {
        ui.set_node_view_cache_flags(boundary, true, true, true);
        ui.nodes[boundary].bounds = Rect::new(
            Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
            Size::new(fret_core::Px(100.0), fret_core::Px(100.0)),
        );
    }

    let scroll_handle = crate::scroll::ScrollHandle::default();
    let handle_key = scroll_handle.binding_key();
    let live_element = crate::GlobalElementId(5);
    let stale_element = crate::GlobalElementId(6);

    crate::declarative::frame::with_window_frame_mut(&mut app, window, |window_frame| {
        for (node, element) in [(live_scroll, live_element), (stale_scroll, stale_element)] {
            window_frame.instances.insert(
                node,
                crate::declarative::frame::ElementRecord {
                    element,
                    instance: crate::declarative::frame::ElementInstance::Scroll(
                        crate::element::ScrollProps {
                            layout: crate::element::LayoutStyle::default(),
                            axis: crate::element::ScrollAxis::Y,
                            scroll_handle: Some(scroll_handle.clone()),
                            known_content_size: None,
                            intrinsic_measure_mode:
                                crate::element::ScrollIntrinsicMeasureMode::Content,
                            windowed_paint: true,
                            probe_unbounded: true,
                        },
                    ),
                    inherited_foreground: None,
                    inherited_text_style: None,
                    semantics_decoration: None,
                    key_context: None,
                    layout_direction: fret_core::LayoutDirection::default(),
                },
            );
        }
    });

    let frame_id = app.frame_id();
    crate::declarative::frame::register_scroll_handle_bindings_batch(
        &mut app,
        window,
        frame_id,
        [crate::declarative::frame::ScrollHandleBinding {
            handle_key,
            element: stale_element,
            handle: scroll_handle.clone(),
        }],
    );
    crate::declarative::frame::register_scroll_handle_bindings_batch(
        &mut app,
        window,
        frame_id,
        [crate::declarative::frame::ScrollHandleBinding {
            handle_key,
            element: live_element,
            handle: scroll_handle.clone(),
        }],
    );

    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );
    for id in [
        root,
        live_boundary,
        live_scroll,
        stale_boundary,
        stale_scroll,
    ] {
        ui.test_clear_node_invalidations(id);
        ui.nodes[id].view_cache_needs_rerender = false;
    }

    scroll_handle.set_offset(fret_core::Point::new(
        fret_core::Px(0.0),
        fret_core::Px(250.0),
    ));
    ui.invalidate_scroll_handle_bindings_for_changed_handles(
        &mut app,
        crate::layout_pass::LayoutPassKind::Final,
        false,
        true,
    );

    assert!(
        !ui.nodes[live_boundary].view_cache_needs_rerender,
        "the attached cache root should repaint without forcing a view-cache rerender"
    );
    assert!(
        ui.nodes[live_boundary].invalidation.paint,
        "the attached cache root should still observe the windowed-paint repaint"
    );
    assert!(
        !ui.nodes[stale_boundary].view_cache_needs_rerender,
        "detached same-frame stale bindings must not dirty detached cache roots"
    );
    assert!(
        !ui.nodes[stale_scroll].invalidation.hit_test,
        "detached stale scroll nodes must not receive scroll-handle invalidations"
    );
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

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(RafWidget);

    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);

    ui.nodes[boundary].view_cache.enabled = true;
    ui.nodes[boundary]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

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
fn view_cache_observation_records_root_observations_as_boundary_subscribers() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);
    let global = TypeId::of::<usize>();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_node_view_cache_flags(boundary, true, true, true);
    let boundary_subscriber = ui
        .test_observation_subscriber_for_boundary_node(boundary)
        .expect("expected boundary subscriber");

    let mut observed = ObservationIndex::default();
    let subscriber = ui.observation_subscriber_for_node(boundary);
    assert_eq!(subscriber, boundary_subscriber);
    observed.record_node_for_subscriber(
        boundary,
        subscriber,
        &[(model.id(), Invalidation::Layout)],
    );

    assert!(observed.by_subscriber.contains_key(&boundary_subscriber));
    assert_eq!(observed.by_subscriber.len(), 1);
    let by_model = observed
        .by_model
        .get(&model.id())
        .expect("expected model observation");
    let mask = by_model
        .get(&boundary_subscriber)
        .copied()
        .expect("expected cache-root observation");
    assert!(mask.layout);
    assert!(mask.paint);
    assert!(!mask.hit_test);

    let mut observed_globals = GlobalObservationIndex::default();
    let subscriber = ui.observation_subscriber_for_node(boundary);
    assert_eq!(subscriber, boundary_subscriber);
    observed_globals.record_node_for_subscriber(
        boundary,
        subscriber,
        &[(global, Invalidation::Paint)],
    );

    assert!(
        observed_globals
            .by_subscriber
            .contains_key(&boundary_subscriber)
    );
    assert_eq!(observed_globals.by_subscriber.len(), 1);
    let by_global = observed_globals
        .by_global
        .get(&global)
        .expect("expected global observation");
    let mask = by_global
        .get(&boundary_subscriber)
        .copied()
        .expect("expected cache-root global observation");
    assert!(mask.paint);
    assert!(!mask.layout);
    assert!(!mask.hit_test);
}

#[test]
fn view_cache_observation_records_descendant_observations_as_boundary_subscribers() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);
    let global = TypeId::of::<usize>();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);
    ui.set_node_view_cache_flags(boundary, true, true, true);
    let boundary_subscriber = ui
        .test_observation_subscriber_for_boundary_node(boundary)
        .expect("expected boundary subscriber");
    let leaf_subscriber = ObservationSubscriber::node(leaf);

    let mut observed = ObservationIndex::default();
    let subscriber = ui.observation_subscriber_for_node(leaf);
    assert_eq!(subscriber, boundary_subscriber);
    observed.record_node_for_subscriber(
        leaf,
        subscriber,
        &[(model.id(), Invalidation::HitTestOnly)],
    );

    assert!(observed.by_subscriber.contains_key(&boundary_subscriber));
    assert!(!observed.by_subscriber.contains_key(&leaf_subscriber));
    let by_model = observed
        .by_model
        .get(&model.id())
        .expect("expected model observation");
    assert!(by_model.contains_key(&boundary_subscriber));
    assert!(!by_model.contains_key(&leaf_subscriber));
    let mask = by_model
        .get(&boundary_subscriber)
        .copied()
        .expect("expected uplifted observation");
    assert!(mask.paint);
    assert!(!mask.layout);
    assert!(mask.hit_test);

    let mut observed_globals = GlobalObservationIndex::default();
    let subscriber = ui.observation_subscriber_for_node(leaf);
    assert_eq!(subscriber, boundary_subscriber);
    observed_globals.record_node_for_subscriber(
        leaf,
        subscriber,
        &[(global, Invalidation::HitTest)],
    );

    assert!(
        observed_globals
            .by_subscriber
            .contains_key(&boundary_subscriber)
    );
    assert!(
        !observed_globals
            .by_subscriber
            .contains_key(&leaf_subscriber)
    );
    let by_global = observed_globals
        .by_global
        .get(&global)
        .expect("expected global observation");
    assert!(by_global.contains_key(&boundary_subscriber));
    assert!(!by_global.contains_key(&leaf_subscriber));
    let mask = by_global
        .get(&boundary_subscriber)
        .copied()
        .expect("expected uplifted global observation");
    assert!(mask.paint);
    assert!(mask.layout);
    assert!(mask.hit_test);
}

#[test]
fn view_cache_boundary_subscriber_observations_merge_root_and_descendant_masks() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);
    let global = TypeId::of::<usize>();

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let boundary = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![boundary]);
    ui.set_children(boundary, vec![leaf]);
    ui.set_node_view_cache_flags(boundary, true, true, true);
    let boundary_subscriber = ui
        .test_observation_subscriber_for_boundary_node(boundary)
        .expect("expected boundary subscriber");
    let leaf_subscriber = ObservationSubscriber::node(leaf);

    let mut observed = ObservationIndex::default();
    let root_subscriber = ui.observation_subscriber_for_node(boundary);
    let descendant_subscriber = ui.observation_subscriber_for_node(leaf);
    assert_eq!(root_subscriber, boundary_subscriber);
    assert_eq!(descendant_subscriber, boundary_subscriber);
    observed.record_node_for_subscriber(
        boundary,
        root_subscriber,
        &[(model.id(), Invalidation::Paint)],
    );
    observed.record_node_for_subscriber(
        leaf,
        descendant_subscriber,
        &[(model.id(), Invalidation::Layout)],
    );

    assert!(!observed.by_subscriber.contains_key(&leaf_subscriber));
    let by_model = observed
        .by_model
        .get(&model.id())
        .expect("expected model observation");
    assert_eq!(by_model.len(), 1);
    let mask = by_model
        .get(&boundary_subscriber)
        .copied()
        .expect("expected merged cache-root observation");
    assert!(mask.paint);
    assert!(mask.layout);
    assert!(!mask.hit_test);
    let by_subscriber = observed
        .by_subscriber
        .get(&boundary_subscriber)
        .expect("expected cache-root observations");
    assert_eq!(by_subscriber.len(), 1);
    assert_eq!(by_subscriber[0].0, model.id());
    assert_eq!(by_subscriber[0].1, mask);

    let mut observed_globals = GlobalObservationIndex::default();
    let root_subscriber = ui.observation_subscriber_for_node(boundary);
    let descendant_subscriber = ui.observation_subscriber_for_node(leaf);
    assert_eq!(root_subscriber, boundary_subscriber);
    assert_eq!(descendant_subscriber, boundary_subscriber);
    observed_globals.record_node_for_subscriber(
        boundary,
        root_subscriber,
        &[(global, Invalidation::Paint)],
    );
    observed_globals.record_node_for_subscriber(
        leaf,
        descendant_subscriber,
        &[(global, Invalidation::HitTestOnly)],
    );

    assert!(
        !observed_globals
            .by_subscriber
            .contains_key(&leaf_subscriber)
    );
    let by_global = observed_globals
        .by_global
        .get(&global)
        .expect("expected global observation");
    assert_eq!(by_global.len(), 1);
    let mask = by_global
        .get(&boundary_subscriber)
        .copied()
        .expect("expected merged cache-root global observation");
    assert!(mask.paint);
    assert!(!mask.layout);
    assert!(mask.hit_test);
    let by_subscriber = observed_globals
        .by_subscriber
        .get(&boundary_subscriber)
        .expect("expected cache-root global observations");
    assert_eq!(by_subscriber.len(), 1);
    assert_eq!(by_subscriber[0].0, global);
    assert_eq!(by_subscriber[0].1, mask);
}

#[test]
fn view_cache_uplifts_observations_to_nearest_root_and_invalidates_ancestor_roots() {
    let mut app = crate::test_host::TestHost::new();
    let model = app.models_mut().insert(0u32);

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let outer = ui.create_node(TestStack);
    let inner = ui.create_node(TestStack);
    let leaf = ui.create_node(PaintObservingWidget {
        model: model.clone(),
    });

    ui.set_root(root);
    ui.set_children(root, vec![outer]);
    ui.set_children(outer, vec![inner]);
    ui.set_children(inner, vec![leaf]);

    ui.nodes[outer].view_cache.enabled = true;
    ui.nodes[outer]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);
    ui.nodes[inner].view_cache.enabled = true;
    ui.nodes[inner]
        .view_cache
        .test_set_layout_contained_when_bounds_known(true);

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
    let inner_subscriber = ui
        .test_observation_subscriber_for_boundary_node(inner)
        .expect("expected inner boundary subscriber");
    let outer_subscriber = ui
        .test_observation_subscriber_for_boundary_node(outer)
        .expect("expected outer boundary subscriber");
    let leaf_subscriber = ObservationSubscriber::node(leaf);
    assert!(
        observed.contains_key(&inner_subscriber),
        "nearest cache root should observe"
    );
    assert!(
        !observed.contains_key(&leaf_subscriber),
        "leaf observation should be uplifted to cache root in view-cache mode"
    );
    assert!(
        !observed.contains_key(&outer_subscriber),
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
