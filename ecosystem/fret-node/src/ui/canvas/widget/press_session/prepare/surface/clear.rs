use crate::ui::canvas::state::InteractionState;

pub(super) fn clear_surface_pointer_sessions(interaction: &mut InteractionState) {
    super::super::super::clear::clear_edge_drag(interaction);
    super::super::super::clear::clear_edge_insert_drag(interaction);
    super::super::super::clear::clear_group_drag(interaction);
    super::super::super::clear::clear_group_resize(interaction);
    super::super::super::clear::clear_node_drag(interaction);
    super::super::super::clear::clear_node_resize(interaction);
    super::super::super::clear::clear_wire_drag(interaction);
    super::super::super::clear::clear_marquee(interaction);
}

pub(super) fn clear_for_group_resize(interaction: &mut InteractionState) {
    super::super::super::clear::clear_group_drag(interaction);
    super::super::super::clear::clear_node_drag(interaction);
    super::super::super::clear::clear_node_resize(interaction);
    super::super::super::clear::clear_wire_drag(interaction);
    super::super::super::clear::clear_edge_drag(interaction);
    super::super::super::clear::clear_edge_insert_drag(interaction);
    super::super::super::clear::clear_marquee(interaction);
}

pub(super) fn clear_for_group_drag(interaction: &mut InteractionState) {
    super::super::super::clear::clear_node_drag(interaction);
    super::super::super::clear::clear_node_resize(interaction);
    super::super::super::clear::clear_wire_drag(interaction);
    super::super::super::clear::clear_edge_drag(interaction);
    super::super::super::clear::clear_edge_insert_drag(interaction);
    super::super::super::clear::clear_marquee(interaction);
}

pub(super) fn clear_for_pan_begin(interaction: &mut InteractionState) {
    super::super::super::clear::clear_group_drag(interaction);
    super::super::super::clear::clear_group_resize(interaction);
    super::super::super::clear::clear_node_drag(interaction);
    super::super::super::clear::clear_node_resize(interaction);
    super::super::super::clear::clear_wire_drag(interaction);
    super::super::super::clear::clear_edge_drag(interaction);
    super::super::super::clear::clear_marquee(interaction);
}
