use fret_core::Point;

use crate::ui::canvas::state::{PendingGroupResize, ViewSnapshot};

pub(super) fn pending_group_resize_threshold_exceeded(
    pending: &PendingGroupResize,
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
