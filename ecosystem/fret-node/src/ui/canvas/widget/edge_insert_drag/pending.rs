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

    super::super::pending_connection_session::activate_pending_edge_insert_drag(
        &mut canvas.interaction,
        pending,
        position,
    );
    invalidate_paint(cx);
    true
}
