mod state;
mod tail;

use super::prelude::*;

pub(in super::super) fn handle_edge_insert_drag_move<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    position: Point,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: CanvasPaintInvalidationCx<H>,
{
    if !state::update_edge_insert_drag_position(&mut canvas.interaction, position) {
        return false;
    }

    tail::finish_edge_insert_drag_move(cx);
    true
}
