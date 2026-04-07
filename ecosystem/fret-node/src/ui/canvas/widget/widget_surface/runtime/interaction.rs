use super::*;

pub(super) fn view_interacting(interaction: &InteractionState) -> bool {
    viewport_interacting(interaction)
        || selection_interacting(interaction)
        || transform_interacting(interaction)
        || connection_interacting(interaction)
        || overlay_interacting(interaction)
}

fn viewport_interacting(interaction: &InteractionState) -> bool {
    interaction.viewport_move_debounce.is_some()
        || interaction.panning
        || interaction.pan_inertia.is_some()
        || interaction.viewport_animation.is_some()
}

fn selection_interacting(interaction: &InteractionState) -> bool {
    interaction.pending_marquee.is_some() || interaction.marquee.is_some()
}

fn transform_interacting(interaction: &InteractionState) -> bool {
    interaction.pending_node_drag.is_some()
        || interaction.node_drag.is_some()
        || interaction.pending_group_drag.is_some()
        || interaction.group_drag.is_some()
        || interaction.pending_group_resize.is_some()
        || interaction.group_resize.is_some()
        || interaction.pending_node_resize.is_some()
        || interaction.node_resize.is_some()
}

fn connection_interacting(interaction: &InteractionState) -> bool {
    interaction.pending_wire_drag.is_some()
        || interaction.wire_drag.is_some()
        || interaction.suspended_wire_drag.is_some()
        || interaction.pending_edge_insert_drag.is_some()
        || interaction.edge_insert_drag.is_some()
        || interaction.edge_drag.is_some()
        || interaction.pending_insert_node_drag.is_some()
        || interaction.insert_node_drag_preview.is_some()
}

fn overlay_interacting(interaction: &InteractionState) -> bool {
    super::super::super::menu_session::has_active_menu_session(interaction)
}
