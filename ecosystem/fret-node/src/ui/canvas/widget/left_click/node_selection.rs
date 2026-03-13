use crate::core::NodeId as GraphNodeId;
use crate::io::NodeGraphViewState;
use crate::ui::canvas::state::PendingNodeSelectAction;

pub(super) fn pending_node_select_action(
    node_selectable: bool,
    multi_selection_pressed: bool,
) -> PendingNodeSelectAction {
    if node_selectable && multi_selection_pressed {
        PendingNodeSelectAction::Toggle
    } else {
        PendingNodeSelectAction::None
    }
}

pub(super) fn apply_node_hit_selection(view_state: &mut NodeGraphViewState, node: GraphNodeId) {
    view_state.selected_edges.clear();
    view_state.selected_groups.clear();
    if !view_state.selected_nodes.iter().any(|id| *id == node) {
        view_state.selected_nodes.clear();
        view_state.selected_nodes.push(node);
    }
    view_state.draw_order.retain(|id| *id != node);
    view_state.draw_order.push(node);
}

#[cfg(test)]
mod tests;
