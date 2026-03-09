use crate::core::{EdgeId, NodeId as GraphNodeId, PortId};
use crate::io::NodeGraphViewState;
use crate::ui::canvas::state::InteractionState;

pub(super) fn clear_focused_port_hints(interaction: &mut InteractionState) {
    interaction.focused_port_valid = false;
    interaction.focused_port_convertible = false;
}

pub(super) fn clear_hover_port_hints(interaction: &mut InteractionState) {
    interaction.hover_port = None;
    interaction.hover_port_valid = false;
    interaction.hover_port_convertible = false;
    interaction.hover_port_diagnostic = None;
}

pub(super) fn focus_edge(interaction: &mut InteractionState, edge: EdgeId) {
    interaction.focused_edge = Some(edge);
    interaction.focused_node = None;
    interaction.focused_port = None;
    clear_focused_port_hints(interaction);
}

pub(super) fn focus_node(interaction: &mut InteractionState, node: GraphNodeId) {
    interaction.focused_node = Some(node);
    interaction.focused_edge = None;
    interaction.focused_port = None;
    clear_focused_port_hints(interaction);
}

pub(super) fn focus_port(interaction: &mut InteractionState, owner: GraphNodeId, port: PortId) {
    interaction.focused_node = Some(owner);
    interaction.focused_edge = None;
    interaction.focused_port = Some(port);
    clear_focused_port_hints(interaction);
}

pub(super) fn select_only_edge(state: &mut NodeGraphViewState, edge: EdgeId) {
    state.selected_nodes.clear();
    state.selected_groups.clear();
    state.selected_edges.clear();
    state.selected_edges.push(edge);
}

pub(super) fn select_only_node(
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
    use crate::ui::canvas::state::InteractionState;

    #[test]
    fn focus_edge_clears_port_focus_and_hints() {
        let mut interaction = InteractionState {
            focused_node: Some(GraphNodeId::from_u128(1)),
            focused_port: Some(PortId::from_u128(2)),
            focused_port_valid: true,
            focused_port_convertible: true,
            ..Default::default()
        };

        let edge = EdgeId::from_u128(3);
        focus_edge(&mut interaction, edge);

        assert_eq!(interaction.focused_edge, Some(edge));
        assert_eq!(interaction.focused_node, None);
        assert_eq!(interaction.focused_port, None);
        assert!(!interaction.focused_port_valid);
        assert!(!interaction.focused_port_convertible);
    }

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
