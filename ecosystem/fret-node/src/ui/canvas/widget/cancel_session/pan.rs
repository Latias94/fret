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
mod tests {
    use super::*;
    use fret_core::{Point, Px};

    #[test]
    fn clear_pan_drag_state_resets_pan_fields() {
        let mut interaction = InteractionState {
            panning: true,
            panning_button: Some(MouseButton::Left),
            pan_last_screen_pos: Some(Point::new(Px(5.0), Px(6.0))),
            pan_last_sample_at: Some(fret_core::time::Instant::now()),
            ..Default::default()
        };

        clear_pan_drag_state(&mut interaction);

        assert!(!interaction.panning);
        assert_eq!(interaction.panning_button, None);
        assert_eq!(interaction.pan_last_screen_pos, None);
        assert_eq!(interaction.pan_last_sample_at, None);
    }
}
