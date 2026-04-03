use super::*;

#[test]
fn set_root_replacement_clears_detached_base_layer_interaction_state() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let old_root = ui.create_node(TestStack);
    let old_leaf = ui.create_node(TestStack);
    ui.add_child(old_root, old_leaf);
    ui.set_root(old_root);

    ui.set_focus(Some(old_leaf));
    ui.captured.insert(fret_core::PointerId(0), old_leaf);

    let new_root = ui.create_node(TestStack);
    ui.set_root(new_root);

    assert_eq!(ui.base_root(), Some(new_root));
    assert_eq!(
        ui.focus(),
        None,
        "replacing the base root must clear focus that is no longer reachable from the active layer roots"
    );
    assert_eq!(
        ui.captured(),
        None,
        "replacing the base root must clear pointer capture that is no longer reachable from the active layer roots"
    );

    let snapshot = ui.input_arbitration_snapshot();
    assert!(
        !snapshot.pointer_capture_active,
        "detached base-layer captures must not remain visible in the arbitration snapshot after root replacement"
    );
    assert_eq!(snapshot.pointer_capture_layer, None);
    assert!(!snapshot.pointer_capture_multiple_layers);
}

#[test]
fn set_root_replacement_preserves_overlay_interaction_state() {
    let mut ui: UiTree<crate::test_host::TestHost> = UiTree::new();
    ui.set_window(AppWindowId::default());

    let old_root = ui.create_node(TestStack);
    ui.set_root(old_root);

    let overlay_root = ui.create_node(TestStack);
    let overlay_leaf = ui.create_node(TestStack);
    ui.add_child(overlay_root, overlay_leaf);
    let overlay_layer = ui.push_overlay_root(overlay_root, false);

    ui.set_focus(Some(overlay_leaf));
    ui.captured.insert(fret_core::PointerId(0), overlay_leaf);

    let new_root = ui.create_node(TestStack);
    ui.set_root(new_root);

    assert_eq!(
        ui.focus(),
        Some(overlay_leaf),
        "root replacement must not clear focus that is still reachable from another active layer root"
    );
    assert_eq!(
        ui.captured(),
        Some(overlay_leaf),
        "root replacement must not clear pointer capture that remains in an active overlay layer"
    );

    let snapshot = ui.input_arbitration_snapshot();
    assert!(snapshot.pointer_capture_active);
    assert_eq!(snapshot.pointer_capture_layer, Some(overlay_layer));
    assert!(!snapshot.pointer_capture_multiple_layers);
}
