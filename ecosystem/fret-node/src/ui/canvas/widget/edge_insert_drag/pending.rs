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
    if canvas.interaction.edge_insert_drag.is_some() {
        return false;
    }
    let Some(pending) = canvas.interaction.pending_edge_insert_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold.max(0.0);
    if !exceeds_drag_threshold(pending.start_pos, position, threshold_screen, snapshot.zoom) {
        return true;
    }

    canvas.interaction.pending_edge_insert_drag = None;
    canvas.interaction.edge_insert_drag = Some(EdgeInsertDrag {
        edge: pending.edge,
        pos: position,
    });
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
