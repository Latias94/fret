use fret_core::Point;

use crate::ui::canvas::state::{PendingGroupDrag, ViewSnapshot};

pub(super) fn pending_group_drag_threshold_exceeded(
    pending: &PendingGroupDrag,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let threshold_screen = snapshot.interaction.node_drag_threshold;
    super::super::threshold::exceeds_drag_threshold(
        pending.start_pos,
        position,
        threshold_screen,
        zoom,
    )
}
