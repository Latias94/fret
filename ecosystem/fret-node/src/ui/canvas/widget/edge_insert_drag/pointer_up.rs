mod active;
mod pending;

use super::prelude::*;

pub(in super::super) fn handle_edge_insert_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
) -> bool {
    if pending::handle_pending_edge_insert_left_up(canvas, cx) {
        return true;
    }

    active::handle_active_edge_insert_left_up(canvas, cx, position)
}
