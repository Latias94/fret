use super::*;

#[test]
fn select_only_node_updates_selection_and_draw_order() {
    let keep = GraphNodeId::from_u128(1);
    let target = GraphNodeId::from_u128(2);
    let mut state = NodeGraphViewState {
        selected_nodes: vec![keep],
        selected_edges: vec![EdgeId::from_u128(9)],
        selected_groups: vec![crate::core::GroupId::from_u128(10)],
        draw_order: vec![target, keep],
        ..Default::default()
    };

    select_only_node(&mut state, target, true);

    assert_eq!(state.selected_nodes, vec![target]);
    assert!(state.selected_edges.is_empty());
    assert!(state.selected_groups.is_empty());
    assert_eq!(state.draw_order.last().copied(), Some(target));
}
