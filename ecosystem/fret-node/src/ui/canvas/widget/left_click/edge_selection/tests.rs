use super::*;
use crate::core::{GroupId, NodeId};

#[test]
fn apply_edge_selection_replaces_selection_without_multi_select() {
    let edge = EdgeId::new();
    let other_edge = EdgeId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_nodes.push(NodeId::new());
    view_state.selected_groups.push(GroupId::new());
    view_state.selected_edges.push(other_edge);

    apply_edge_selection(&mut view_state, edge, false);

    assert!(view_state.selected_nodes.is_empty());
    assert!(view_state.selected_groups.is_empty());
    assert_eq!(view_state.selected_edges, vec![edge]);
}

#[test]
fn apply_edge_selection_toggles_edge_in_multi_select_mode() {
    let edge = EdgeId::new();
    let other_edge = EdgeId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_edges.push(other_edge);

    apply_edge_selection(&mut view_state, edge, true);
    assert_eq!(view_state.selected_edges, vec![other_edge, edge]);

    apply_edge_selection(&mut view_state, edge, true);
    assert_eq!(view_state.selected_edges, vec![other_edge]);
}

#[test]
fn focused_edge_after_hit_requires_focusable_and_selectable() {
    let edge = EdgeId::new();
    assert_eq!(focused_edge_after_hit(true, true, edge), Some(edge));
    assert_eq!(focused_edge_after_hit(false, true, edge), None);
    assert_eq!(focused_edge_after_hit(true, false, edge), None);
}
