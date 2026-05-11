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

#[test]
fn suppressed_parent_does_not_aggregate_dirty_child_transitions() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let gate = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![gate]);
    ui.set_children(gate, vec![child]);

    ui.test_clear_node_invalidations(child);
    ui.test_clear_node_invalidations(gate);
    ui.test_clear_node_invalidations(root);

    ui.set_layout_dirty_children_suppressed(gate, true);

    ui.test_set_layout_invalidation(child, true);

    assert_eq!(ui.nodes[child].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);

    ui.test_set_layout_invalidation(gate, true);

    assert_eq!(ui.nodes[child].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);

    ui.test_set_layout_invalidation(child, false);

    assert_eq!(ui.nodes[child].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);
}

#[test]
fn remove_dirty_child_from_suppressed_parent_does_not_underflow_or_repair() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let gate = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![gate]);
    ui.set_children(gate, vec![child]);

    ui.test_clear_node_invalidations(child);
    ui.test_clear_node_invalidations(gate);
    ui.test_clear_node_invalidations(root);

    ui.test_set_layout_invalidation(child, true);
    ui.set_layout_dirty_children_suppressed(gate, true);

    assert_eq!(ui.nodes[child].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);

    let rebuilds_before = ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes;
    let mut services = FakeUiServices;
    let removed = ui.remove_subtree(&mut services, child);

    assert_eq!(removed, vec![child]);
    assert!(!ui.nodes.contains_key(child));
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);
    assert_eq!(
        ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes,
        rebuilds_before,
        "removing a dirty child hidden behind a suppressed parent must not trip the underflow repair path"
    );
}
