use crate::ui::canvas::state::InteractionState;

pub(super) fn allow_close_button_cursor(
    has_close_command: bool,
    interaction: &InteractionState,
) -> bool {
    super::interaction_gate::allow_close_button_cursor(has_close_command, interaction)
}

pub(super) fn allow_canvas_detail_cursor(interaction: &InteractionState) -> bool {
    super::interaction_gate::allow_canvas_detail_cursor(interaction)
}

#[cfg(test)]
mod tests;
