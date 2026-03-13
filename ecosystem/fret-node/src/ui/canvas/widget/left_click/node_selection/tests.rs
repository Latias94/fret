use super::*;
use crate::core::{EdgeId, GroupId};

#[test]
fn pending_node_select_action_requires_selectable_and_multi_select() {
    assert!(matches!(
        pending_node_select_action(true, true),
        PendingNodeSelectAction::Toggle
    ));
    assert!(matches!(
        pending_node_select_action(false, true),
        PendingNodeSelectAction::None
    ));
    assert!(matches!(
        pending_node_select_action(true, false),
        PendingNodeSelectAction::None
    ));
}

#[test]
fn apply_node_hit_selection_clears_other_selection_and_elevates_draw_order() {
    let node = GraphNodeId::new();
    let other = GraphNodeId::new();
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_edges.push(EdgeId::new());
    view_state.selected_groups.push(GroupId::new());
    view_state.selected_nodes.push(other);
    view_state.draw_order.extend([node, other]);

    apply_node_hit_selection(&mut view_state, node);

    assert!(view_state.selected_edges.is_empty());
    assert!(view_state.selected_groups.is_empty());
    assert_eq!(view_state.selected_nodes, vec![node]);
    assert_eq!(view_state.draw_order, vec![other, node]);
}
