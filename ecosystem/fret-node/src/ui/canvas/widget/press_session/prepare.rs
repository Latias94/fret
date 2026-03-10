use crate::ui::canvas::state::InteractionState;

use super::clear;

pub(in super::super) fn prepare_for_port_hit(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_edge_anchor_hit(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_hover_port_hints(interaction);
    interaction.hover_edge = None;
}

pub(in super::super) fn prepare_for_resize_hit(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_node_hit(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_edge_hit(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    super::super::focus_session::clear_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_group_resize(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_group_drag(interaction: &mut InteractionState) {
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_background_interaction(interaction: &mut InteractionState) {
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_selection_marquee(interaction: &mut InteractionState) {
    clear::clear_edge_drag(interaction);
    clear::clear_edge_insert_drag(interaction);
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super) fn prepare_for_pan_begin(interaction: &mut InteractionState) {
    clear::clear_group_drag(interaction);
    clear::clear_group_resize(interaction);
    clear::clear_node_drag(interaction);
    clear::clear_node_resize(interaction);
    clear::clear_wire_drag(interaction);
    clear::clear_edge_drag(interaction);
    clear::clear_marquee(interaction);
    super::super::focus_session::clear_hover_edge_focus_and_hover_port_hints(interaction);
}
