use super::*;
use crate::core::GroupId;

#[test]
fn select_group_context_target_clears_node_and_edge_selection() {
    let group_id = GroupId::new();
    let mut view_state = super::test_support::view_state_with_node_and_edge();

    group::select_group_context_target_in_view_state(&mut view_state, group_id);

    assert!(view_state.selected_nodes.is_empty());
    assert!(view_state.selected_edges.is_empty());
    assert_eq!(view_state.selected_groups, vec![group_id]);
}

#[test]
fn select_group_context_target_preserves_existing_group_if_already_selected() {
    let group_id = GroupId::new();
    let other_group = GroupId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_groups.extend([group_id, other_group]);

    group::select_group_context_target_in_view_state(&mut view_state, group_id);

    assert_eq!(view_state.selected_groups, vec![group_id, other_group]);
}

#[test]
fn select_edge_context_target_clears_node_and_group_selection() {
    let edge_id = EdgeId::new();
    let mut view_state = super::test_support::view_state_with_node_and_group();

    edge::select_edge_context_target_in_view_state(&mut view_state, edge_id);

    assert!(view_state.selected_nodes.is_empty());
    assert!(view_state.selected_groups.is_empty());
    assert_eq!(view_state.selected_edges, vec![edge_id]);
}

#[test]
fn select_edge_context_target_preserves_existing_edge_if_already_selected() {
    let edge_id = EdgeId::new();
    let other_edge = EdgeId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_edges.extend([edge_id, other_edge]);

    edge::select_edge_context_target_in_view_state(&mut view_state, edge_id);

    assert_eq!(view_state.selected_edges, vec![edge_id, other_edge]);
}
