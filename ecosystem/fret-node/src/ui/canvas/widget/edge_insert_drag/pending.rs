mod activate;
mod checks;

use super::prelude::*;

pub(in super::super) fn handle_pending_edge_insert_drag_move<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
) -> bool {
    let pending = match checks::prepare_pending_edge_insert_drag_move(canvas, snapshot, position) {
        checks::PendingEdgeInsertDragMovePrep::NotHandled => return false,
        checks::PendingEdgeInsertDragMovePrep::Handled => return true,
        checks::PendingEdgeInsertDragMovePrep::Ready(pending) => pending,
    };

    activate::activate_pending_edge_insert_drag(canvas, cx, pending, position);
    true
}
