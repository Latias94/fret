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

#[test]
fn set_children_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let parent = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![parent]);
    ui.set_children(parent, vec![child]);

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(120.0), fret_core::Px(60.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [root, parent, child] {
        ui.test_clear_node_invalidations(id);
    }

    ui.test_set_node_parent(child, None);
    ui.test_set_layout_invalidation(child, true);

    assert!(ui.nodes[child].invalidation.layout);
    assert!(!ui.nodes[parent].invalidation.layout);
    assert!(!ui.nodes[root].invalidation.layout);

    ui.set_children(parent, vec![child]);

    assert_eq!(ui.node_parent(child), Some(parent));
    assert!(ui.nodes[parent].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !ui.nodes[child].invalidation.layout,
        "repairing same-children parent pointers must reconnect detached descendant layout invalidations to the authoritative layout pass"
    );
}

#[test]
fn set_children_in_mount_same_children_repairs_parent_pointers_and_reconnects_dirty_descendant_layout()
 {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let parent = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![parent]);
    ui.set_children_in_mount(parent, vec![child]);

    let bounds = Rect::new(
        Point::new(fret_core::Px(0.0), fret_core::Px(0.0)),
        Size::new(fret_core::Px(120.0), fret_core::Px(60.0)),
    );
    let mut services = FakeUiServices;
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    for id in [root, parent, child] {
        ui.test_clear_node_invalidations(id);
    }

    ui.test_set_node_parent(child, None);
    ui.test_set_layout_invalidation(child, true);

    assert!(ui.nodes[child].invalidation.layout);
    assert!(!ui.nodes[parent].invalidation.layout);
    assert!(!ui.nodes[root].invalidation.layout);

    ui.set_children_in_mount(parent, vec![child]);

    assert_eq!(ui.node_parent(child), Some(parent));
    assert!(ui.nodes[parent].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !ui.nodes[child].invalidation.layout,
        "mount-time same-children parent repair must reconnect detached descendant layout invalidations to the authoritative layout pass"
    );
}

#[test]
fn add_child_reparents_from_old_parent_without_leaving_stale_child_edges() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let left = ui.create_node(TestStack);
    let right = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![left, right]);
    ui.set_children(left, vec![child]);

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(left);
    ui.test_clear_node_invalidations(right);
    ui.test_clear_node_invalidations(child);

    ui.add_child(right, child);

    assert_eq!(ui.node_parent(child), Some(right));
    assert_eq!(ui.nodes[left].children, Vec::<NodeId>::new());
    assert_eq!(ui.nodes[right].children, vec![child]);
    assert!(ui.nodes[left].invalidation.layout);
    assert!(ui.nodes[right].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);
}

#[test]
fn add_child_noops_when_child_is_already_attached_once_to_same_parent() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.add_child(root, child);

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(child);

    ui.add_child(root, child);

    assert_eq!(ui.node_parent(child), Some(root));
    assert_eq!(ui.nodes[root].children, vec![child]);
    assert!(!ui.nodes[root].invalidation.hit_test);
    assert!(!ui.nodes[root].invalidation.layout);
    assert!(!ui.nodes[root].invalidation.paint);
}
