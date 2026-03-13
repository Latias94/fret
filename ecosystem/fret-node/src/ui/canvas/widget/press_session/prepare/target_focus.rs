use crate::ui::canvas::state::InteractionState;
use crate::ui::canvas::widget::focus_session;

use super::clear;

pub(in super::super::super::super) fn prepare_for_port_hit(interaction: &mut InteractionState) {
    clear::clear_for_port_hit(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_edge_anchor_hit(
    interaction: &mut InteractionState,
) {
    clear::clear_for_edge_anchor_hit(interaction);
    focus_session::clear_hover_port_hints(interaction);
    interaction.hover_edge = None;
}

pub(in super::super::super::super) fn prepare_for_resize_hit(interaction: &mut InteractionState) {
    clear::clear_for_resize_hit(interaction);
    focus_session::clear_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_node_hit(interaction: &mut InteractionState) {
    clear::clear_for_node_hit(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_edge_hit(interaction: &mut InteractionState) {
    clear::clear_for_edge_hit(interaction);
    focus_session::clear_hover_port_hints(interaction);
}
