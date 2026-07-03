use super::*;

#[test]
fn propagation_depth_uses_child_edges_under_stale_parent_pointers() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();

    let root = ui.create_node(TestStack);
    let actual_parent = ui.create_node(TestStack);
    let leaf = ui.create_node(TestStack);

    ui.set_root(root);
    ui.add_child(root, actual_parent);
    ui.add_child(actual_parent, leaf);

    ui.test_set_node_parent(leaf, None);

    assert_eq!(
        crate::tree::propagation_depth::propagation_depth_for(&mut ui, leaf),
        2,
        "propagation depth must follow current child-edge topology, not retained parent pointers"
    );
}
