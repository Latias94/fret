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
}
