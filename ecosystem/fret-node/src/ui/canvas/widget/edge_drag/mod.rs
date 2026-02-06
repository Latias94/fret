mod move_start;
mod pointer_up;
mod prelude;

use prelude::*;

pub(super) fn handle_edge_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    move_start::handle_edge_drag_move(canvas, cx, snapshot, position, zoom)
}

pub(super) fn handle_edge_left_up<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    pointer_up::handle_edge_left_up(canvas, cx)
}
