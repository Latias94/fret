use fret_core::Point;

use super::super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::{PendingEdgeInsertDrag, ViewSnapshot};

pub(super) enum PendingEdgeInsertDragMovePrep {
    NotHandled,
    Handled,
    Ready(PendingEdgeInsertDrag),
}

pub(super) fn prepare_pending_edge_insert_drag_move<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> PendingEdgeInsertDragMovePrep {
    if canvas.interaction.edge_insert_drag.is_some() {
        return PendingEdgeInsertDragMovePrep::NotHandled;
    }
    let Some(pending) = canvas.interaction.pending_edge_insert_drag.clone() else {
        return PendingEdgeInsertDragMovePrep::NotHandled;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold.max(0.0);
    if !should_activate_pending_edge_insert_drag(
        pending.start_pos,
        position,
        threshold_screen,
        snapshot.zoom,
    ) {
        return PendingEdgeInsertDragMovePrep::Handled;
    }

    PendingEdgeInsertDragMovePrep::Ready(pending)
}

fn should_activate_pending_edge_insert_drag(
    start_pos: Point,
    position: Point,
    threshold_screen: f32,
    zoom: f32,
) -> bool {
    super::super::super::threshold::exceeds_drag_threshold(
        start_pos,
        position,
        threshold_screen,
        zoom,
    )
}

#[cfg(test)]
mod tests;
