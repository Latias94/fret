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
mod tests {
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
}
