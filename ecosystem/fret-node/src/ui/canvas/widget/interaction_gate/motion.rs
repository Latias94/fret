use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn pan_inertia_should_tick(interaction: &InteractionState) -> bool {
    if super::super::menu_session::has_active_menu_session(interaction) {
        return false;
    }
    if interaction.panning {
        return false;
    }
    interaction.pending_marquee.is_none()
        && interaction.marquee.is_none()
        && interaction.pending_node_drag.is_none()
        && interaction.node_drag.is_none()
        && interaction.pending_group_drag.is_none()
        && interaction.group_drag.is_none()
        && interaction.pending_group_resize.is_none()
        && interaction.group_resize.is_none()
        && interaction.pending_node_resize.is_none()
        && interaction.node_resize.is_none()
        && interaction.pending_wire_drag.is_none()
        && interaction.wire_drag.is_none()
        && interaction.edge_drag.is_none()
}

#[cfg(test)]
mod tests;
