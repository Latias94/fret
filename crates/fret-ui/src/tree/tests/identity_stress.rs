use super::*;

#[test]
fn element_index_resolves_live_node_without_retained_scan() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_001);
    let root = ui.create_node(TestStack);
    let live_node = ui.create_node_for_element(element, TestStack);

    ui.set_root(root);
    ui.add_child(root, live_node);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, None),
        Some(live_node)
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_index_hits, 1);
    assert_eq!(stats.identity_resolve_index_stale, 0);
    assert_eq!(stats.identity_resolve_index_misses, 0);
}

#[test]
fn seeded_stale_resolution_uses_index_without_retained_scan() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_002);
    let root = ui.create_node(TestStack);
    let live_node = ui.create_node_for_element(element, TestStack);
    let stale_detached = ui.create_node_for_element(element, TestStack);

    ui.set_root(root);
    ui.add_child(root, live_node);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, Some(stale_detached)),
        Some(live_node)
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_seeded_stale, 1);
    assert_eq!(stats.identity_resolve_index_hits, 1);
}

#[test]
fn element_index_rejects_rebound_old_element_binding() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let old_element = crate::elements::GlobalElementId(10_003);
    let new_element = crate::elements::GlobalElementId(10_004);
    let root = ui.create_node(TestStack);
    let node = ui.create_node_for_element(old_element, TestStack);

    ui.set_root(root);
    ui.add_child(root, node);
    ui.set_node_element(node, Some(new_element));

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(old_element, None),
        None,
        "old element binding must not resolve after the node is rebound"
    );
    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(new_element, None),
        Some(node),
        "new element binding should resolve through the updated index"
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_index_misses, 1);
    assert_eq!(stats.identity_resolve_index_hits, 1);
}

#[test]
fn duplicate_live_element_ids_are_diagnostic_instead_of_silent_overwrite() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_005);
    let root = ui.create_node(TestStack);
    let first = ui.create_node_for_element(element, TestStack);
    let second = ui.create_node_for_element(element, TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![first, second]);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, None),
        None,
        "duplicate live declarative ids must not silently pick a retained node"
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_index_duplicate_live, 1);
}

#[test]
fn seeded_duplicate_live_element_ids_are_still_diagnostic() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_007);
    let root = ui.create_node(TestStack);
    let first = ui.create_node_for_element(element, TestStack);
    let second = ui.create_node_for_element(element, TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![first, second]);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, Some(first)),
        Some(first)
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_seeded_hits, 1);
    assert_eq!(
        stats.identity_resolve_index_duplicate_live, 1,
        "a valid seed must not hide duplicate live declarative ids"
    );
}

#[test]
fn removed_node_seed_cannot_resolve_after_new_binding_reuses_element() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_008);
    let root = ui.create_node(TestStack);
    let removed_node = ui.create_node_for_element(element, TestStack);

    ui.set_root(root);
    ui.add_child(root, removed_node);

    let mut services = FakeUiServices;
    let removed = ui.remove_subtree(&mut services, removed_node);
    assert_eq!(removed, vec![removed_node]);

    ui.begin_debug_frame_if_needed(FrameId(1));

    let new_node = ui.create_node_for_element(element, TestStack);
    ui.add_child(root, new_node);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, Some(removed_node)),
        Some(new_node),
        "a removed stale seed must not satisfy the element after a new binding lands"
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_seeded_stale, 1);
    assert_eq!(stats.identity_resolve_index_hits, 1);
}

#[test]
fn indexed_detached_handle_is_stale_even_when_binding_generation_matches() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

    let element = crate::elements::GlobalElementId(10_009);
    let detached = ui.create_node_for_element(element, TestStack);

    // Simulate a missed detach cleanup. The live index must still validate attachment before
    // returning a node, so a stale handle cannot become an authoritative live hit.
    ui.index_node_element_binding(detached, element);

    assert_eq!(
        ui.resolve_live_attached_node_for_element_seeded(element, None),
        None
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.identity_resolve_index_stale, 1);
    assert_eq!(stats.identity_resolve_index_hits, 0);
    assert_eq!(stats.identity_resolve_index_misses, 0);
}

#[test]
fn element_index_isolated_per_ui_tree_window() {
    let element = crate::elements::GlobalElementId(10_006);

    let mut ui_a: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui_a.set_window(AppWindowId::from(KeyData::from_ffi(101)));
    ui_a.set_debug_enabled(true);
    let root_a = ui_a.create_node(TestStack);
    let node_a = ui_a.create_node_for_element(element, TestStack);
    ui_a.set_root(root_a);
    ui_a.add_child(root_a, node_a);

    let mut ui_b: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui_b.set_window(AppWindowId::from(KeyData::from_ffi(102)));
    ui_b.set_debug_enabled(true);
    let root_b = ui_b.create_node(TestStack);
    let node_b = ui_b.create_node_for_element(element, TestStack);
    ui_b.set_root(root_b);
    ui_b.add_child(root_b, node_b);

    assert_eq!(
        ui_a.resolve_live_attached_node_for_element_seeded(element, None),
        Some(node_a)
    );
    assert_eq!(
        ui_b.resolve_live_attached_node_for_element_seeded(element, None),
        Some(node_b)
    );

    let stats_a = ui_a.debug_stats();
    let stats_b = ui_b.debug_stats();
    assert_eq!(stats_a.identity_resolve_index_hits, 1);
    assert_eq!(stats_b.identity_resolve_index_hits, 1);
    assert_eq!(stats_a.identity_resolve_index_duplicate_live, 0);
    assert_eq!(stats_b.identity_resolve_index_duplicate_live, 0);
}
