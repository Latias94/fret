use crate::ui::canvas::state::InteractionState;
use crate::ui::canvas::widget::focus_session;

use super::clear;

pub(in super::super::super::super) fn prepare_for_group_resize(interaction: &mut InteractionState) {
    clear::clear_for_group_resize(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_group_drag(interaction: &mut InteractionState) {
    clear::clear_for_group_drag(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_background_interaction(
    interaction: &mut InteractionState,
) {
    clear::clear_surface_pointer_sessions(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_selection_marquee(
    interaction: &mut InteractionState,
) {
    clear::clear_surface_pointer_sessions(interaction);
    focus_session::clear_edge_focus_and_hover_port_hints(interaction);
}

pub(in super::super::super::super) fn prepare_for_pan_begin(interaction: &mut InteractionState) {
    clear::clear_for_pan_begin(interaction);
    focus_session::clear_hover_edge_focus_and_hover_port_hints(interaction);
}
