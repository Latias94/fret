use crate::core::{EdgeId, GroupId, NodeId as GraphNodeId};
use crate::ui::canvas::widget::NodeGraphViewState;

pub(super) fn view_state_with_node_and_edge() -> NodeGraphViewState {
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_nodes.push(GraphNodeId::new());
    view_state.selected_edges.push(EdgeId::new());
    view_state
}

pub(super) fn view_state_with_node_and_group() -> NodeGraphViewState {
    let mut view_state = NodeGraphViewState::default();
    view_state.selected_nodes.push(GraphNodeId::new());
    view_state.selected_groups.push(GroupId::new());
    view_state
}
