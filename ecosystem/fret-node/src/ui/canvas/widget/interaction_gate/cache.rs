use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn allow_edges_cache(interaction: &InteractionState) -> bool {
    interaction.pending_wire_drag.is_none()
        && interaction.wire_drag.is_none()
        && interaction.suspended_wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && interaction.pending_insert_node_drag.is_none()
        && interaction.insert_node_drag_preview.is_none()
}

#[cfg(test)]
mod tests;
