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
mod tests;
