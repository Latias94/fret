use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_cancel_residuals(interaction: &mut InteractionState) -> bool {
    let mut canceled = false;

    if interaction.pending_right_click.take().is_some() {
        canceled = true;
    }
    if interaction.sticky_wire || interaction.sticky_wire_ignore_next_up {
        interaction.sticky_wire = false;
        interaction.sticky_wire_ignore_next_up = false;
        canceled = true;
    }
    if interaction.snap_guides.take().is_some() {
        canceled = true;
    }

    canceled
}

pub(in super::super) fn clear_hover_edge_focus(interaction: &mut InteractionState) {
    super::super::focus_session::clear_hover_port_hints(interaction);
    interaction.hover_edge = None;
    interaction.hover_edge_anchor = None;
    interaction.focused_edge = None;
}

#[cfg(test)]
mod tests;
