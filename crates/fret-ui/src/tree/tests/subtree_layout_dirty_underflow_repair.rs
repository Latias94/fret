use super::*;

#[test]
fn subtree_layout_dirty_underflow_repairs_counts_upwards() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let a = ui.create_node(TestStack);
    let b = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![a, b]);

    ui.test_clear_node_invalidations(a);
    ui.test_clear_node_invalidations(b);
    ui.test_clear_node_invalidations(root);

    ui.test_set_layout_invalidation(b, true);
    assert!(ui.nodes[b].invalidation.layout);
    assert_eq!(ui.nodes[b].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);

    // Simulate drift: we're about to apply a `true -> false` layout transition for `a`, but both
    // `a` and its ancestors have already lost the corresponding aggregated counts.
    ui.nodes[a].subtree_layout_dirty_count = 0;
    ui.nodes[root].subtree_layout_dirty_count = 0;
    ui.nodes[a].invalidation.layout = false;

    ui.note_layout_invalidation_transition_for_subtree_aggregation(a, true, false);

    assert_eq!(ui.nodes[a].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[b].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);
}
