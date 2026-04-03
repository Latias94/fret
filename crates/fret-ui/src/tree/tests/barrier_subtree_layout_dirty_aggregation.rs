use super::*;

#[test]
fn set_children_barrier_updates_subtree_layout_dirty_counts_for_dirty_children() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let barrier = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![barrier]);

    ui.test_clear_node_invalidations(barrier);
    ui.test_clear_node_invalidations(root);

    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);

    let child = ui.create_node(TestStack);
    ui.set_children_barrier(barrier, vec![child]);

    assert_eq!(ui.node_parent(child), Some(barrier));
    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 2);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 2);
}

#[test]
fn schedule_barrier_relayout_updates_subtree_layout_dirty_counts() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let barrier = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![barrier]);

    ui.test_clear_node_invalidations(barrier);
    ui.test_clear_node_invalidations(root);

    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);

    ui.schedule_barrier_relayout_with_source_and_detail(
        barrier,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::Unknown,
    );

    assert!(ui.nodes[barrier].invalidation.layout);
    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);
}

#[test]
fn set_children_barrier_same_children_recomputes_subtree_layout_dirty_counts() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let barrier = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![barrier]);
    ui.set_children_barrier(barrier, vec![child]);

    ui.test_clear_node_invalidations(child);
    ui.test_clear_node_invalidations(barrier);
    ui.test_clear_node_invalidations(root);

    ui.test_set_layout_invalidation(child, true);
    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);

    // Simulate drift: the barrier subtree count was lost, but a caller re-applies the same child
    // list (common for virtualization barriers where the visible window is unchanged).
    ui.nodes[barrier].subtree_layout_dirty_count = 0;
    ui.nodes[root].subtree_layout_dirty_count = 0;

    ui.set_children_barrier(barrier, vec![child]);

    assert_eq!(ui.nodes[barrier].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);
}

#[test]
fn detached_pending_barrier_relayout_is_pruned_before_layout() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let barrier = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![barrier]);

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(120.0), fret_core::Px(60.0)),
    );
    ui.nodes[root].bounds = bounds;
    ui.nodes[root].measured_size = bounds.size;
    ui.nodes[barrier].bounds = bounds;
    ui.nodes[barrier].measured_size = bounds.size;

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(barrier);

    ui.schedule_barrier_relayout_with_source_and_detail(
        barrier,
        UiDebugInvalidationSource::Other,
        UiDebugInvalidationDetail::Unknown,
    );
    ui.set_children(root, Vec::new());
    assert_eq!(ui.node_parent(barrier), None);

    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.debug_stats().barrier_relayouts_performed,
        0,
        "detached barrier roots must not keep running pending barrier relayouts"
    );
}
