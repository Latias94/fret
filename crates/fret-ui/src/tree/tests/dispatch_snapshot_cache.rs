use super::*;

#[test]
fn dispatch_snapshot_cache_reuses_forest_across_frames_until_structure_changes() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
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

    let child_b = ui.create_node(TestStack);
    ui.set_children(root, vec![child_a, child_b]);
    let (active_roots, barrier_root) = ui.active_input_layers();
    let snapshot_c = ui.cached_dispatch_snapshot_for_layer_roots(
        FrameId(3),
        active_roots.as_slice(),
        barrier_root,
    );

    assert!(!Arc::ptr_eq(&snapshot_a.nodes, &snapshot_c.nodes));
    assert!(snapshot_c.pre.get(child_b).is_some());
}
