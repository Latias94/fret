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
