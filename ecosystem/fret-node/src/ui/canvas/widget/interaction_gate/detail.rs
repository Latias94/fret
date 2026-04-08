use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn allow_close_button_cursor(
    has_close_command: bool,
    interaction: &InteractionState,
) -> bool {
    has_close_command && allows_pointer_detail_gestures(interaction)
}

pub(in super::super) fn allow_canvas_detail_cursor(interaction: &InteractionState) -> bool {
    allows_pointer_detail_gestures(interaction)
        && interaction.marquee.is_none()
        && !super::super::menu_session::has_active_menu_session(interaction)
}

fn allows_pointer_detail_gestures(interaction: &InteractionState) -> bool {
    interaction.node_drag.is_none()
        && interaction.node_resize.is_none()
        && interaction.wire_drag.is_none()
        && interaction.pending_edge_insert_drag.is_none()
        && interaction.edge_insert_drag.is_none()
        && interaction.edge_drag.is_none()
        && !interaction.panning
}

#[cfg(test)]
mod tests;
