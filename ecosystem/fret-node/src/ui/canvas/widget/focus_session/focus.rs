use crate::core::{EdgeId, NodeId as GraphNodeId, PortId};
use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn focus_edge(interaction: &mut InteractionState, edge: EdgeId) {
    interaction.focused_edge = Some(edge);
    interaction.focused_node = None;
    interaction.focused_port = None;
    super::hints::clear_focused_port_hints(interaction);
}

pub(in super::super) fn focus_node(interaction: &mut InteractionState, node: GraphNodeId) {
    interaction.focused_node = Some(node);
    interaction.focused_edge = None;
    interaction.focused_port = None;
    super::hints::clear_focused_port_hints(interaction);
}

pub(in super::super) fn focus_port(
    interaction: &mut InteractionState,
    owner: GraphNodeId,
    port: PortId,
) {
    interaction.focused_node = Some(owner);
    interaction.focused_edge = None;
    interaction.focused_port = Some(port);
    super::hints::clear_focused_port_hints(interaction);
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
