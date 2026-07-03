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
fn set_children_same_children_records_parent_drift_without_global_repair_and_reconnects_layout() {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

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
    let would_repair = ui.parent_pointers_would_repair_from_layer_roots();
    ui.debug_record_parent_pointer_would_repair(would_repair);
    assert_eq!(
        ui.node_parent(child),
        None,
        "shadow oracle must not mutate retained parent pointers"
    );

    let stats = ui.debug_stats();
    assert_eq!(stats.parent_pointer_would_repair_passes, 1);
    assert_eq!(stats.parent_pointer_would_repair_nodes, 1);
    assert_eq!(stats.parent_pointer_repair_passes, 0);
    assert_eq!(stats.parent_pointer_repairs, 0);

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
        "same-children parent edge sync must reconnect detached descendant layout invalidations to the authoritative layout pass"
    );
}

#[test]
fn set_children_in_mount_same_children_syncs_parent_edge_without_global_repair_and_reconnects_layout()
 {
    let mut app = crate::test_host::TestHost::new();
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);

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
    assert_eq!(ui.debug_stats().parent_pointer_repair_passes, 0);
    assert_eq!(ui.debug_stats().parent_pointer_repairs, 0);

    app.advance_frame();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(
        !ui.nodes[child].invalidation.layout,
        "mount-time same-children parent edge sync must reconnect detached descendant layout invalidations to the authoritative layout pass"
    );
}

#[test]
fn set_children_in_mount_new_dirty_layer_root_skips_redundant_structural_walk() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());
    ui.set_debug_enabled(true);
    ui.begin_debug_frame_if_needed(FrameId(1));

    let parent = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);
    ui.set_root(parent);

    assert!(ui.nodes[parent].invalidation.layout);
    assert!(ui.nodes[parent].invalidation.paint);
    assert!(ui.nodes[parent].invalidation.hit_test);
    assert_eq!(ui.node_parent(parent), None);
    assert_eq!(ui.node_parent(child), None);
    assert_eq!(ui.nodes[parent].subtree_layout_dirty_count, 1);

    let walks_before = ui.debug_invalidation_walks().len();

    ui.set_children_in_mount(parent, vec![child]);

    assert_eq!(ui.node_parent(child), Some(parent));
    assert_eq!(ui.nodes[parent].children, vec![child]);
    assert!(ui.nodes[parent].invalidation.layout);
    assert!(ui.nodes[parent].invalidation.paint);
    assert!(ui.nodes[parent].invalidation.hit_test);
    assert_eq!(ui.nodes[parent].subtree_layout_dirty_count, 2);

    let new_walks = &ui.debug_invalidation_walks()[walks_before..];
    assert!(
        new_walks.iter().all(|w| {
            w.detail != UiDebugInvalidationDetail::StructuralChildrenChanged || w.root != parent
        }),
        "new mount-time dirty parent should not emit a redundant structural invalidation walk; walks={new_walks:?}"
    );
}

#[test]
fn set_children_in_mount_stale_retained_none_parent_does_not_skip_live_ancestor_walk() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let ancestor = ui.create_node(TestStack);
    let parent = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![ancestor]);
    ui.set_children(ancestor, vec![parent]);

    for id in [root, ancestor, parent, child] {
        ui.test_clear_node_invalidations(id);
    }
    ui.invalidate(parent, Invalidation::HitTest);
    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(ancestor);
    ui.test_set_node_parent(parent, None);

    assert_eq!(
        ui.node_parent_in_layer_tree(parent),
        Some(ancestor),
        "test setup must keep the authoritative child-edge parent"
    );
    assert_eq!(
        ui.node_parent(parent),
        None,
        "test setup must simulate stale retained parent storage"
    );

    ui.set_children_in_mount(parent, vec![child]);

    assert!(
        ui.nodes[ancestor].invalidation.hit_test,
        "live non-root mount changes must propagate through child-edge ancestors"
    );
    assert!(
        ui.nodes[root].invalidation.hit_test,
        "stale retained parent storage must not trigger the initial layer-root fast path"
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

#[test]
fn set_children_reparents_from_old_parent_without_leaving_stale_child_edges() {
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

    ui.set_children(right, vec![child]);

    assert_eq!(ui.node_parent(child), Some(right));
    assert_eq!(ui.nodes[left].children, Vec::<NodeId>::new());
    assert_eq!(ui.nodes[right].children, vec![child]);
    assert!(ui.nodes[left].invalidation.layout);
    assert!(ui.nodes[right].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);
}

#[test]
fn set_children_in_mount_reparents_from_old_parent_without_leaving_stale_child_edges() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let left = ui.create_node(TestStack);
    let right = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![left, right]);
    ui.set_children_in_mount(left, vec![child]);

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(left);
    ui.test_clear_node_invalidations(right);
    ui.test_clear_node_invalidations(child);

    ui.set_children_in_mount(right, vec![child]);

    assert_eq!(ui.node_parent(child), Some(right));
    assert_eq!(ui.nodes[left].children, Vec::<NodeId>::new());
    assert_eq!(ui.nodes[right].children, vec![child]);
    assert!(ui.nodes[left].invalidation.layout);
    assert!(ui.nodes[right].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);
}

#[test]
fn set_children_barrier_reparents_from_old_barrier_without_leaving_stale_child_edges() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let left = ui.create_node(TestStack);
    let right = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![left, right]);
    ui.set_children_barrier(left, vec![child]);
    ui.take_pending_barrier_relayouts();

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(left);
    ui.test_clear_node_invalidations(right);
    ui.test_clear_node_invalidations(child);

    ui.set_children_barrier(right, vec![child]);

    assert_eq!(ui.node_parent(child), Some(right));
    assert_eq!(ui.nodes[left].children, Vec::<NodeId>::new());
    assert_eq!(ui.nodes[right].children, vec![child]);
    assert!(
        !ui.nodes[root].invalidation.layout,
        "barrier reparent should not force ancestor relayout through the old barrier parent"
    );

    let pending = ui.take_pending_barrier_relayouts();
    assert_eq!(pending, vec![left, right]);
}

#[test]
fn set_children_reparents_from_old_barrier_using_barrier_detach_semantics() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let root = ui.create_node(TestStack);
    let left = ui.create_node(TestStack);
    let right = ui.create_node(TestStack);
    let child = ui.create_node(TestStack);

    ui.set_root(root);
    ui.set_children(root, vec![left, right]);
    ui.set_children_barrier(left, vec![child]);
    ui.take_pending_barrier_relayouts();

    ui.test_clear_node_invalidations(root);
    ui.test_clear_node_invalidations(left);
    ui.test_clear_node_invalidations(right);
    ui.test_clear_node_invalidations(child);

    ui.set_children(right, vec![child]);

    assert_eq!(ui.node_parent(child), Some(right));
    assert_eq!(ui.nodes[left].children, Vec::<NodeId>::new());
    assert_eq!(ui.nodes[right].children, vec![child]);
    assert!(ui.nodes[right].invalidation.layout);
    assert!(ui.nodes[root].invalidation.layout);

    let pending = ui.take_pending_barrier_relayouts();
    assert_eq!(
        pending,
        vec![left],
        "detaching from a barrier parent must preserve contained relayout scheduling on the old parent"
    );
}
