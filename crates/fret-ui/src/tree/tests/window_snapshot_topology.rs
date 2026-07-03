use super::*;

#[test]
fn pending_declarative_snapshot_commit_uses_child_edges_under_stale_parent_pointers() {
    let mut app = crate::test_host::TestHost::new();
    app.set_global(fret_runtime::PlatformCapabilities::default());

    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let pending_root = ui.create_node(TestStack);

    ui.set_root(root);
    ui.defer_declarative_window_snapshot_commit(pending_root);
    ui.set_children(root, vec![pending_root]);
    ui.test_set_node_parent(pending_root, None);

    assert!(
        ui.commit_pending_declarative_window_runtime_snapshots(&mut app, pending_root),
        "pending declarative snapshot commits must use child-edge attachment, not retained parent pointers"
    );
}
