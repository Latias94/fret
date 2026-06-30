use super::*;

#[test]
fn prepaint_interaction_cache_replays_for_clean_view_cache_root() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);

    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(ui.debug_stats().interaction_cache_hits, 0);

    app.advance_frame();
    // Force a non-stable layout pass so the test exercises interaction-cache replay.
    //
    // The layout engine can legitimately skip work on a completely stable frame, which would
    // bypass prepaint recording/replay and make `interaction_cache_hits` remain 0.
    ui.invalidate(root, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let stats = ui.debug_stats();
    assert!(stats.interaction_cache_hits >= 1);
    assert!(stats.interaction_cache_replayed_records > 0);
}

#[test]
fn prepaint_interaction_cache_entry_is_owned_by_view_boundary_state() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        ui.test_view_boundary_interaction_cache_has_entry(cache_root),
        "view-cache roots should store interaction replay entries in ViewBoundaryState"
    );
    let boundary = ui
        .debug_boundary_stats()
        .into_iter()
        .find(|boundary| boundary.id == cache_root)
        .expect("cache root boundary stats");
    assert_eq!(
        boundary.interaction_cache_owner,
        "view_boundary_interaction_cache_state"
    );
}

#[test]
fn prepaint_interaction_cache_replay_translates_records_when_cache_root_moves() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    ui.nodes[root].bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    ui.nodes[cache_root].bounds =
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[leaf].bounds = ui.nodes[cache_root].bounds;

    let mut services = FakeUiServices;
    let inputs = PrepaintAfterLayoutInputs::new(&mut services, 1.0);
    ui.prepaint_after_layout(&mut app, inputs);
    assert_eq!(ui.debug_stats().interaction_cache_hits, 0);
    for node in [root, cache_root, leaf] {
        ui.test_clear_node_invalidations(node);
    }
    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(10.0), Px(10.0))),
        Some(leaf)
    );

    app.advance_frame();
    ui.clear_hit_test_path_cache();
    let moved_bounds = Rect::new(
        Point::new(Px(100.0), Px(0.0)),
        Size::new(Px(20.0), Px(20.0)),
    );
    ui.nodes[cache_root].bounds = moved_bounds;
    ui.nodes[leaf].bounds = moved_bounds;
    for node in [root, cache_root, leaf] {
        ui.test_clear_node_invalidations(node);
    }

    let inputs = PrepaintAfterLayoutInputs::new(&mut services, 1.0);
    ui.prepaint_after_layout(&mut app, inputs);
    let stats = ui.debug_stats();
    assert!(
        stats.interaction_cache_hits >= 1,
        "expected clean cache root to reuse interaction records after an origin-only move (stats={stats:?})"
    );

    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(110.0), Px(10.0))),
        Some(leaf),
        "replayed interaction records must move with the cache root"
    );
    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(10.0), Px(10.0))),
        Some(root),
        "old cache-root position should no longer hit the moved cached subtree"
    );
}

#[test]
fn prepaint_interaction_cache_root_move_invalidates_stale_root_only_hit_path() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    ui.nodes[root].bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(200.0), Px(100.0)),
    );
    ui.nodes[cache_root].bounds =
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[leaf].bounds = ui.nodes[cache_root].bounds;

    let mut services = FakeUiServices;
    let inputs = PrepaintAfterLayoutInputs::new(&mut services, 1.0);
    ui.prepaint_after_layout(&mut app, inputs);
    for node in [root, cache_root, leaf] {
        ui.test_clear_node_invalidations(node);
    }

    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(60.0), Px(10.0))),
        Some(root),
        "prime a root-only cached path at a point where the cached child is absent"
    );

    app.advance_frame();
    let moved_bounds = Rect::new(Point::new(Px(50.0), Px(0.0)), Size::new(Px(20.0), Px(20.0)));
    ui.nodes[cache_root].bounds = moved_bounds;
    ui.nodes[leaf].bounds = moved_bounds;
    for node in [root, cache_root, leaf] {
        ui.test_clear_node_invalidations(node);
    }

    let inputs = PrepaintAfterLayoutInputs::new(&mut services, 1.0);
    ui.prepaint_after_layout(&mut app, inputs);
    let stats = ui.debug_stats();
    assert!(
        stats.interaction_cache_hits >= 1,
        "expected cache-root movement to reuse translated interaction records (stats={stats:?})"
    );

    let stats_before_rehit = ui.debug_stats();
    assert_eq!(
        ui.hit_test_layers_cached(&[root], Point::new(Px(60.0), Px(10.0))),
        Some(leaf),
        "a stale root-only path cache must not hide a cache-root child that moved under the pointer"
    );
    let stats_after_rehit = ui.debug_stats();
    assert!(
        stats_after_rehit.hit_test_path_cache_misses
            > stats_before_rehit.hit_test_path_cache_misses,
        "expected the stale root-only path cache to miss and force a full hit-test before accepting the moved child"
    );
}

struct PrepaintCountStack {
    calls: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl<H: UiHost> Widget<H> for PrepaintCountStack {
    fn prepaint(&mut self, _cx: &mut crate::widget::PrepaintCx<'_, H>) {
        use std::sync::atomic::Ordering;
        self.calls.fetch_add(1, Ordering::SeqCst);
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }
}

#[test]
fn prepaint_hook_runs_for_view_cache_root_even_when_reusing_interaction_cache() {
    use std::sync::atomic::Ordering;

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(PrepaintCountStack {
        calls: calls.clone(),
    });
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);

    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(ui.debug_stats().interaction_cache_hits, 0);

    app.advance_frame();
    // Force a non-stable layout pass so the test exercises interaction-cache replay (and ensures
    // the view-cache root's prepaint hook still runs under reuse).
    ui.invalidate(root, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(ui.debug_stats().interaction_cache_hits >= 1);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn prepaint_hook_runs_for_manual_cache_root_when_global_view_cache_is_disabled() {
    use std::sync::atomic::Ordering;

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(false);

    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(PrepaintCountStack {
        calls: calls.clone(),
    });
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);

    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    app.advance_frame();
    ui.invalidate(root, Invalidation::Layout);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

struct PrepaintActionStack;

impl<H: UiHost> Widget<H> for PrepaintActionStack {
    fn prepaint(&mut self, cx: &mut crate::widget::PrepaintCx<'_, H>) {
        cx.invalidate_self(crate::widget::Invalidation::Paint);
        cx.invalidate_self(crate::widget::Invalidation::HitTestOnly);
        cx.request_redraw();
        cx.request_animation_frame();
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }
}

#[test]
fn prepaint_actions_are_exported_to_debug_snapshot() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(PrepaintActionStack);
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let actions = ui.debug_prepaint_actions();
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::Invalidate),
        "expected at least one prepaint invalidate action"
    );
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::RequestRedraw),
        "expected at least one prepaint request_redraw action"
    );
    assert!(
        actions
            .iter()
            .any(|a| a.kind == crate::tree::UiDebugPrepaintActionKind::RequestAnimationFrame),
        "expected at least one prepaint request_animation_frame action"
    );
}

struct PrepaintOutputCounter {
    seen_prev: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl<H: UiHost> Widget<H> for PrepaintOutputCounter {
    fn prepaint(&mut self, cx: &mut crate::widget::PrepaintCx<'_, H>) {
        use std::sync::atomic::Ordering;
        let prev = cx.output::<u32>().copied().unwrap_or(0);
        if prev > 0 {
            self.seen_prev.fetch_add(1, Ordering::SeqCst);
        }
        cx.set_output(prev.saturating_add(1));
    }

    fn layout(&mut self, cx: &mut LayoutCx<'_, H>) -> Size {
        for &child in cx.children {
            let _ = cx.layout_in(child, cx.bounds);
        }
        cx.available
    }
}

#[test]
fn prepaint_output_store_is_keyed_by_cache_root_prepaint_key() {
    use std::sync::atomic::Ordering;

    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);
    ui.set_debug_enabled(true);

    let seen_prev = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(PrepaintOutputCounter {
        seen_prev: seen_prev.clone(),
    });
    let leaf = ui.create_node(TestStack);
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.add_child(cache_root, leaf);
    ui.set_node_view_cache_flags(cache_root, true, false, false);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(seen_prev.load(Ordering::SeqCst), 0);
    assert_eq!(ui.prepaint_output::<u32>(cache_root).copied(), Some(1));

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(seen_prev.load(Ordering::SeqCst), 1);
    assert_eq!(ui.prepaint_output::<u32>(cache_root).copied(), Some(2));

    // Changing the scale factor changes the cache root's prepaint key, so the output store should
    // reset.
    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 2.0);
    assert_eq!(seen_prev.load(Ordering::SeqCst), 1);
    assert_eq!(ui.prepaint_output::<u32>(cache_root).copied(), Some(1));
}

#[test]
fn prepaint_output_is_owned_by_view_boundary_state_and_removed_with_node() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_view_cache_enabled(true);

    let root = ui.create_node(TestStack);
    let cache_root = ui.create_node(PrepaintOutputCounter {
        seen_prev: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)),
    });
    ui.set_root(root);
    ui.add_child(root, cache_root);
    ui.set_node_view_cache_flags(cache_root, true, true, true);

    let mut services = FakeUiServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(100.0), Px(100.0)),
    );

    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(ui.test_view_boundary_exists(cache_root));
    assert_eq!(ui.test_view_boundary_parent(cache_root), None);
    assert_eq!(
        ui.test_view_boundary_prepaint_output::<u32>(cache_root)
            .copied(),
        Some(1)
    );
    assert_eq!(
        ui.debug_boundary_prepaint_owner_for_node(cache_root),
        "view_boundary_prepaint_state"
    );
    assert!(ui.test_view_boundary_allows_contained_relayout(cache_root));

    let removed = ui.remove_subtree(&mut services, cache_root);
    assert_eq!(removed, vec![cache_root]);
    assert!(!ui.test_view_boundary_exists(cache_root));
    assert_eq!(
        ui.debug_boundary_prepaint_owner_for_node(cache_root),
        "none"
    );
}
