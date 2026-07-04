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
fn subtree_layout_dirty_underflow_uses_child_edges_without_parent_repair() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let a = ui.create_node(TestStack);
    let b = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![a, b]);

    ui.test_clear_node_invalidations(a);
    ui.test_clear_node_invalidations(b);
    ui.test_clear_node_invalidations(root);

    ui.test_set_layout_invalidation(b, true);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);

    ui.nodes[a].subtree_layout_dirty_count = 0;
    ui.nodes[root].subtree_layout_dirty_count = 0;
    ui.nodes[a].invalidation.layout = false;
    ui.test_set_node_parent(a, None);

    ui.note_layout_invalidation_transition_for_subtree_aggregation(a, true, false);

    assert_eq!(ui.nodes[a].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[b].subtree_layout_dirty_count, 1);
    assert_eq!(
        ui.nodes[root].subtree_layout_dirty_count, 1,
        "underflow repair must recompute real child-edge ancestors even when retained parents are stale"
    );
    assert_eq!(
        ui.debug_stats().parent_pointer_repair_passes,
        0,
        "layout dirty underflow must not repair retained parent pointers in the normal path"
    );
    assert_eq!(ui.debug_node_parent_storage(a), None);
}

#[test]
fn semantics_dirty_propagation_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let parent = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![parent]);
    ui.set_children(parent, vec![leaf]);
    ui.clear_all_semantics_dirty_tracking();
    ui.semantics_dirty = false;
    ui.semantics_dirty_all = false;
    ui.test_set_node_parent(leaf, None);

    ui.mark_semantics_dirty_for_node(leaf);

    assert_eq!(ui.nodes[leaf].subtree_semantics_dirty_count, 1);
    assert_eq!(
        ui.nodes[parent].subtree_semantics_dirty_count, 1,
        "semantics dirty propagation must follow child-edge ancestors"
    );
    assert_eq!(ui.nodes[root].subtree_semantics_dirty_count, 1);

    ui.clear_semantics_dirty_nodes(vec![leaf]);

    assert_eq!(ui.nodes[leaf].subtree_semantics_dirty_count, 0);
    assert_eq!(ui.nodes[parent].subtree_semantics_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_semantics_dirty_count, 0);
    assert_eq!(ui.debug_node_parent_storage(leaf), None);
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

#[test]
fn remove_subtree_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let actual_parent = ui.create_node(TestStack);
    let stale_parent = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![actual_parent, stale_parent]);
    ui.set_children(actual_parent, vec![child]);

    ui.test_clear_node_invalidations(child);
    ui.test_clear_node_invalidations(actual_parent);
    ui.test_clear_node_invalidations(stale_parent);
    ui.test_clear_node_invalidations(root);
    ui.test_set_layout_invalidation(child, true);

    assert_eq!(ui.nodes[actual_parent].subtree_layout_dirty_count, 1);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 1);

    ui.test_set_node_parent(child, Some(stale_parent));

    let mut services = FakeUiServices;
    let removed = ui.remove_subtree(&mut services, child);

    assert_eq!(removed, vec![child]);
    assert!(!ui.nodes.contains_key(child));
    assert!(
        !ui.nodes[actual_parent].children.contains(&child),
        "remove_subtree must unlink from the actual child-edge parent"
    );
    assert_eq!(ui.nodes[actual_parent].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[stale_parent].subtree_layout_dirty_count, 0);
    assert_eq!(ui.nodes[root].subtree_layout_dirty_count, 0);
    assert_eq!(ui.debug_stats().parent_pointer_repair_passes, 0);
}

#[test]
fn removing_dirty_subtree_under_suppressed_parent_does_not_decrement_ancestors() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let root = ui.create_node(TestStack);
    let gate = ui.create_node(TestStack);
    let hidden_child = ui.create_node(TestStack);
    let visible_dirty_sibling = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![gate, visible_dirty_sibling]);
    ui.set_children(gate, vec![hidden_child]);

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(gate);
    ui.test_clear_node_invalidations(hidden_child);
    ui.test_clear_node_invalidations(visible_dirty_sibling);

    ui.set_layout_dirty_children_suppressed(gate, true);
    ui.test_set_layout_invalidation(hidden_child, true);
    ui.test_set_layout_invalidation(visible_dirty_sibling, true);

    assert_eq!(
        ui.nodes[hidden_child].subtree_layout_dirty_count, 1,
        "hidden dirty child tracks its own pending layout work"
    );
    assert_eq!(
        ui.nodes[gate].subtree_layout_dirty_count, 0,
        "suppressed parent must not expose hidden child dirty work"
    );
    assert_eq!(
        ui.nodes[root].subtree_layout_dirty_count, 1,
        "root should only count the visible dirty sibling"
    );

    let rebuilds_before = ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes;
    let mut services = FakeUiServices;
    ui.remove_subtree(&mut services, hidden_child);

    assert!(!ui.nodes.contains_key(hidden_child));
    assert_eq!(
        ui.nodes[root].subtree_layout_dirty_count, 1,
        "removing hidden dirty work must not subtract from ancestors that never counted it"
    );
    assert_eq!(ui.nodes[gate].subtree_layout_dirty_count, 0);
    assert_eq!(
        ui.nodes[visible_dirty_sibling].subtree_layout_dirty_count,
        1
    );
    assert_eq!(
        ui.debug_stats().layout_subtree_dirty_agg_rebuild_nodes,
        rebuilds_before,
        "the normal removal path should stay count-consistent without underflow repair"
    );
}
