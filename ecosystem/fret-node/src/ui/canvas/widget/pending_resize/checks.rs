use fret_core::Point;

use crate::ui::canvas::state::{PendingNodeResize, ViewSnapshot};

pub(super) fn pending_node_resize_threshold_exceeded(
    pending: &PendingNodeResize,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let threshold_screen = snapshot.interaction.node_drag_threshold;
    should_activate_pending_node_resize(pending.start_pos, position, threshold_screen, zoom)
}

fn should_activate_pending_node_resize(
    start_pos: Point,
    position: Point,
    threshold_screen: f32,
    zoom: f32,
) -> bool {
    super::super::threshold::exceeds_drag_threshold(start_pos, position, threshold_screen, zoom)
}

#[cfg(test)]
mod tests;
