#[cfg(feature = "diagnostics")]
#[test]
fn removed_subtree_reachability_prefers_frame_context() {
    use super::{FakeUiServices, TestStack};
    use crate::tree::{UiDebugRemoveSubtreeFrameContext, UiTree};
    use fret_core::AppWindowId;
    use fret_runtime::FrameId;

    let _app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_debug_enabled(true);
    ui.begin_debug_frame_if_needed(FrameId(1));

    let root = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);
    ui.set_root(root);

    // Intentionally omit `ui.set_children(root, ..)` so `UiTree` reachability cannot discover
    // `child` via layer-root traversal. The frame context is the authoritative source for
    // "liveness reachability" in remove-subtree diagnostics.
    ui.debug_set_remove_subtree_frame_context(
        child,
        UiDebugRemoveSubtreeFrameContext {
            parent_frame_children_len: None,
            parent_frame_children_contains_root: None,
            root_frame_instance_present: true,
            root_frame_children_len: None,
            root_reachable_from_layer_roots: true,
            root_reachable_from_view_cache_roots: None,
            liveness_layer_roots_len: 0,
            view_cache_reuse_roots_len: 0,
            view_cache_reuse_root_nodes_len: 0,
            trigger_element: None,
            trigger_element_root: None,
            trigger_element_in_view_cache_keep_alive: None,
            trigger_element_listed_under_reuse_root: None,
            path_edge_len: 0,
            path_edge_frame_contains_child: [2u8; 16],
        },
    );

    let mut services = FakeUiServices;
    ui.remove_subtree(&mut services, child);

    let record = ui
        .debug_removed_subtrees()
        .iter()
        .rev()
        .find(|r| r.root == child)
        .expect("expected removed-subtree record");
    assert!(
        record.reachable_from_layer_roots,
        "expected remove-subtree reachability to prefer frame context over UiTree-only traversal"
    );
}
