use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn allow_edge_hover_anchor(interaction: &InteractionState) -> bool {
    interaction.wire_drag.is_none()
        && interaction.insert_node_drag_preview.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && interaction.node_drag.is_none()
        && interaction.node_resize.is_none()
        && interaction.group_drag.is_none()
        && interaction.group_resize.is_none()
        && interaction.marquee.is_none()
        && !super::super::menu_session::has_active_menu_session(interaction)
}

#[cfg(test)]
mod tests;
