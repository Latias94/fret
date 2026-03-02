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
