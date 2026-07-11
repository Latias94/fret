use super::*;

#[test]
fn active_input_barrier_only_reports_visible_blocking_layers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let base = ui.create_node(TestStack);
    let overlay = ui.create_node(TestStack);
    ui.set_root(base);

    assert!(!ui.has_active_input_barrier());
    let layer = ui.push_overlay_root(overlay, true);
    assert!(ui.has_active_input_barrier());

    ui.set_layer_visible(layer, false);
    assert!(!ui.has_active_input_barrier());
}

#[test]
fn active_input_barrier_scope_accepts_modal_and_higher_layer_elements_only() {
    let mut app = crate::test_host::TestHost::new();
    let window = AppWindowId::default();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(window);

    let underlay_element = crate::elements::GlobalElementId(0xA11);
    let underlay = ui.create_node_for_element(underlay_element, TestStack);
    ui.set_root(underlay);
    assert!(!ui.element_is_within_active_input_barrier_scope(&mut app, underlay_element));

    let barrier = ui.create_node(TestStack);
    let modal_element = crate::elements::GlobalElementId(0xB22);
    let modal_child = ui.create_node_for_element(modal_element, TestStack);
    ui.add_child(barrier, modal_child);
    ui.push_overlay_root(barrier, true);

    let higher_layer_element = crate::elements::GlobalElementId(0xC33);
    let higher_layer = ui.create_node_for_element(higher_layer_element, TestStack);
    ui.push_overlay_root(higher_layer, false);

    assert!(!ui.element_is_within_active_input_barrier_scope(&mut app, underlay_element));
    assert!(ui.element_is_within_active_input_barrier_scope(&mut app, modal_element));
    assert!(ui.element_is_within_active_input_barrier_scope(&mut app, higher_layer_element));
    assert!(!ui.element_is_within_active_input_barrier_scope(
        &mut app,
        crate::elements::GlobalElementId(0xDEAD),
    ));
}

#[test]
fn dispatch_snapshot_cache_reuses_forest_across_frames_until_structure_changes() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_debug_enabled(true);
    let root = ui.create_node(TestStack);
    let child_a = ui.create_node(TestStack);
    ui.set_root(root);
    ui.set_children(root, vec![child_a]);

    let (active_roots, barrier_root) = ui.active_input_layers();
    let snapshot_a = ui.cached_dispatch_snapshot_for_layer_roots(
        FrameId(1),
        active_roots.as_slice(),
        barrier_root,
    );
    let snapshot_b = ui.cached_dispatch_snapshot_for_layer_roots(
        FrameId(2),
        active_roots.as_slice(),
        barrier_root,
    );

    assert_eq!(snapshot_b.frame_id, FrameId(2));
    assert!(Arc::ptr_eq(&snapshot_a.nodes, &snapshot_b.nodes));
    assert!(Arc::ptr_eq(&snapshot_a.parent, &snapshot_b.parent));
    assert!(Arc::ptr_eq(&snapshot_a.pre, &snapshot_b.pre));
    assert!(Arc::ptr_eq(&snapshot_a.post, &snapshot_b.post));
    assert_eq!(snapshot_a.topology_epoch, ui.live_topology_epoch());
    assert_eq!(snapshot_b.topology_epoch, snapshot_a.topology_epoch);
    assert_eq!(
        ui.debug_stats().live_topology_epoch,
        snapshot_a.topology_epoch.as_u64()
    );
    let stats = ui.debug_stats();
    assert_eq!(stats.dispatch_snapshot_cache_misses, 1);
    assert_eq!(stats.dispatch_snapshot_cache_hits, 1);
    assert_eq!(stats.dispatch_snapshot_builds, 1);
    assert_eq!(stats.dispatch_snapshot_built_nodes, 2);
    let initial_generation = ui.test_dispatch_snapshot_frame_product_generation();
    assert!(
        initial_generation > 0,
        "structure setup should advance the dispatch snapshot frame-product generation"
    );
    assert_eq!(
        ui.test_dispatch_snapshot_frame_product_cached_entries(),
        1,
        "dispatch snapshot cache entries should be owned by the frame-product state"
    );

    ui.set_children(root, vec![child_a]);
    let snapshot_same_topology = ui.cached_dispatch_snapshot_for_layer_roots(
        FrameId(3),
        active_roots.as_slice(),
        barrier_root,
    );
    assert!(Arc::ptr_eq(
        &snapshot_a.nodes,
        &snapshot_same_topology.nodes
    ));
    assert_eq!(
        snapshot_same_topology.topology_epoch,
        snapshot_a.topology_epoch
    );

    let child_b = ui.create_node(TestStack);
    ui.set_children(root, vec![child_a, child_b]);
    let (active_roots, barrier_root) = ui.active_input_layers();
    let snapshot_c = ui.cached_dispatch_snapshot_for_layer_roots(
        FrameId(4),
        active_roots.as_slice(),
        barrier_root,
    );

    assert!(!Arc::ptr_eq(&snapshot_a.nodes, &snapshot_c.nodes));
    assert!(snapshot_c.topology_epoch > snapshot_a.topology_epoch);
    assert_eq!(snapshot_c.topology_epoch, ui.live_topology_epoch());
    assert!(snapshot_c.pre.get(child_b).is_some());
    let stats = ui.debug_stats();
    assert!(stats.dispatch_snapshot_cache_invalidations >= 1);
    assert_eq!(stats.dispatch_snapshot_cache_misses, 2);
    assert_eq!(stats.dispatch_snapshot_cache_hits, 2);
    assert_eq!(stats.dispatch_snapshot_builds, 2);
    assert_eq!(stats.dispatch_snapshot_built_nodes, 5);
    assert!(
        ui.test_dispatch_snapshot_frame_product_generation() > initial_generation,
        "structure invalidation should advance the dispatch snapshot frame-product generation"
    );
    assert_eq!(
        ui.test_dispatch_snapshot_frame_product_cached_entries(),
        1,
        "structure invalidation should clear stale frame-product snapshots before rebuilding"
    );
}
