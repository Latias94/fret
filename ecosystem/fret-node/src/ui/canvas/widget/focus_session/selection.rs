use crate::core::{EdgeId, NodeId as GraphNodeId};
use crate::io::NodeGraphViewState;

pub(in super::super) fn select_only_edge(state: &mut NodeGraphViewState, edge: EdgeId) {
    state.selected_nodes.clear();
    state.selected_groups.clear();
    state.selected_edges.clear();
    state.selected_edges.push(edge);
}

pub(in super::super) fn select_only_node(
    state: &mut NodeGraphViewState,
    node: GraphNodeId,
    bring_to_front: bool,
) {
    state.selected_edges.clear();
    state.selected_groups.clear();
    state.selected_nodes.clear();
    state.selected_nodes.push(node);
    if bring_to_front {
        state.draw_order.retain(|id| *id != node);
        state.draw_order.push(node);
    }
}

#[cfg(test)]
mod tests;
