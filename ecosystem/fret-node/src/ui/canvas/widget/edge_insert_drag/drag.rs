mod state;
mod tail;

use super::prelude::*;

pub(in super::super) fn handle_edge_insert_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
) -> bool {
    if !state::update_edge_insert_drag_position(&mut canvas.interaction, position) {
        return false;
    }

    tail::finish_edge_insert_drag_move(cx);
    true
}
