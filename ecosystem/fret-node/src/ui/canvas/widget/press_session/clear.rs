use crate::ui::canvas::state::InteractionState;

pub(super) fn clear_group_drag(interaction: &mut InteractionState) {
    interaction.pending_group_drag = None;
    interaction.group_drag = None;
}

pub(super) fn clear_group_resize(interaction: &mut InteractionState) {
    interaction.pending_group_resize = None;
    interaction.group_resize = None;
}

pub(super) fn clear_node_drag(interaction: &mut InteractionState) {
    interaction.pending_node_drag = None;
    interaction.node_drag = None;
}

pub(super) fn clear_node_resize(interaction: &mut InteractionState) {
    interaction.pending_node_resize = None;
    interaction.node_resize = None;
}

pub(super) fn clear_wire_drag(interaction: &mut InteractionState) {
    interaction.pending_wire_drag = None;
    interaction.wire_drag = None;
    interaction.click_connect = false;
}

pub(super) fn clear_edge_drag(interaction: &mut InteractionState) {
    interaction.edge_drag = None;
}

pub(super) fn clear_edge_insert_drag(interaction: &mut InteractionState) {
    interaction.pending_edge_insert_drag = None;
    interaction.edge_insert_drag = None;
}

pub(super) fn clear_marquee(interaction: &mut InteractionState) {
    interaction.pending_marquee = None;
    interaction.marquee = None;
}
