use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_focused_port_hints(interaction: &mut InteractionState) {
    interaction.focused_port_valid = false;
    interaction.focused_port_convertible = false;
}

pub(in super::super) fn clear_hover_port_hints(interaction: &mut InteractionState) {
    interaction.hover_port = None;
    interaction.hover_port_valid = false;
    interaction.hover_port_convertible = false;
    interaction.hover_port_diagnostic = None;
}

pub(in super::super) fn clear_edge_focus(interaction: &mut InteractionState) {
    interaction.focused_edge = None;
}

pub(in super::super) fn clear_edge_focus_and_hover_port_hints(interaction: &mut InteractionState) {
    clear_edge_focus(interaction);
    clear_hover_port_hints(interaction);
}

pub(in super::super) fn clear_hover_edge_focus_and_hover_port_hints(
    interaction: &mut InteractionState,
) {
    interaction.hover_edge = None;
    clear_edge_focus_and_hover_port_hints(interaction);
}

#[cfg(test)]
mod tests;
