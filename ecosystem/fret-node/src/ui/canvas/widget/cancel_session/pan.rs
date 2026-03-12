use fret_core::MouseButton;

use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_pan_drag_state(interaction: &mut InteractionState) {
    interaction.panning = false;
    interaction.panning_button = None;
    interaction.pan_last_screen_pos = None;
    interaction.pan_last_sample_at = None;
}

pub(in super::super) fn matches_pan_release(
    interaction: &InteractionState,
    button: MouseButton,
) -> bool {
    interaction.panning && interaction.panning_button == Some(button)
}

#[cfg(test)]
mod tests;
