use fret_core::Point;

use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingWireDrag, ViewSnapshot};

pub(super) enum PendingWireDragMovePrep {
    NotHandled,
    Handled,
    Ready(PendingWireDrag),
}

pub(super) fn prepare_pending_wire_drag_move<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> PendingWireDragMovePrep {
    if canvas.interaction.wire_drag.is_some() {
        return PendingWireDragMovePrep::NotHandled;
    }
    let Some(pending) = canvas.interaction.pending_wire_drag.clone() else {
        return PendingWireDragMovePrep::NotHandled;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold.max(0.0);
    if !should_activate_pending_wire_drag(pending.start_pos, position, threshold_screen, zoom) {
        return PendingWireDragMovePrep::Handled;
    }

    PendingWireDragMovePrep::Ready(pending)
}

fn should_activate_pending_wire_drag(
    start_pos: Point,
    position: Point,
    threshold_screen: f32,
    zoom: f32,
) -> bool {
    super::super::threshold::exceeds_drag_threshold(start_pos, position, threshold_screen, zoom)
}

#[cfg(test)]
mod tests;
