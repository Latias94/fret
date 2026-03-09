use fret_core::MouseButton;

use crate::ui::canvas::state::InteractionState;

pub(super) fn clear_cancel_residuals(interaction: &mut InteractionState) -> bool {
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

pub(super) fn clear_hover_edge_focus(interaction: &mut InteractionState) {
    super::focus_session::clear_hover_port_hints(interaction);
    interaction.hover_edge = None;
    interaction.hover_edge_anchor = None;
    interaction.focused_edge = None;
}

pub(super) fn clear_pan_drag_state(interaction: &mut InteractionState) {
    interaction.panning = false;
    interaction.panning_button = None;
    interaction.pan_last_screen_pos = None;
    interaction.pan_last_sample_at = None;
}

pub(super) fn matches_pan_release(interaction: &InteractionState, button: MouseButton) -> bool {
    interaction.panning && interaction.panning_button == Some(button)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::{Point, Px};

    #[test]
    fn clear_cancel_residuals_clears_pending_flags() {
        let mut interaction = InteractionState {
            pending_right_click: Some(crate::ui::canvas::state::PendingRightClick {
                start_pos: Point::new(Px(1.0), Px(2.0)),
            }),
            sticky_wire: true,
            sticky_wire_ignore_next_up: true,
            snap_guides: Some(Default::default()),
            ..Default::default()
        };

        assert!(clear_cancel_residuals(&mut interaction));
        assert!(interaction.pending_right_click.is_none());
        assert!(!interaction.sticky_wire);
        assert!(!interaction.sticky_wire_ignore_next_up);
        assert!(interaction.snap_guides.is_none());
    }

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
