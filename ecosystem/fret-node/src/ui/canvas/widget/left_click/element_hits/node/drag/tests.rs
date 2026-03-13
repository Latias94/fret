use super::*;

#[test]
fn base_drag_nodes_uses_multi_selection_only_for_selected_draggable_node() {
    let node = GraphNodeId::new();
    let other = GraphNodeId::new();
    assert_eq!(
        base_drag_nodes(node, true, true, true, &[node, other]),
        vec![node, other]
    );
    assert_eq!(
        base_drag_nodes(node, true, true, false, &[node, other]),
        vec![node]
    );
    assert_eq!(
        base_drag_nodes(node, false, true, true, &[node, other]),
        vec![node]
    );
}

#[test]
fn drag_enabled_for_node_hit_respects_handle_mode() {
    assert!(drag_enabled_for_node_hit(
        NodeGraphDragHandleMode::Any,
        false,
        true
    ));
    assert!(!drag_enabled_for_node_hit(
        NodeGraphDragHandleMode::Header,
        false,
        true
    ));
    assert!(!drag_enabled_for_node_hit(
        NodeGraphDragHandleMode::Any,
        true,
        false
    ));
}
