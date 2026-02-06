use super::*;

#[test]
fn set_children_noops_when_unchanged() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let a = ui.create_node(TestStack);
    let b = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![a, b]);
    ui.test_clear_node_invalidations(root);

    ui.set_children(root, vec![a, b]);

    assert_eq!(ui.node_parent(a), Some(root));
    assert_eq!(ui.node_parent(b), Some(root));

    let inv = &ui.nodes[root].invalidation;
    assert!(!inv.hit_test);
    assert!(!inv.layout);
    assert!(!inv.paint);
}

#[test]
fn set_children_invalidates_parent_when_changed() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let a = ui.create_node(TestStack);
    let b = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![a, b]);
    ui.test_clear_node_invalidations(root);

    ui.set_children(root, vec![b, a]);

    assert_eq!(ui.node_parent(a), Some(root));
    assert_eq!(ui.node_parent(b), Some(root));
    assert!(ui.nodes[root].invalidation.hit_test);
    assert!(ui.nodes[root].invalidation.layout);
    assert!(ui.nodes[root].invalidation.paint);
}
