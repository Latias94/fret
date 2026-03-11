mod catalog;
mod overlay;

use super::*;

pub(super) fn open_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    at: CanvasPoint,
) {
    let request = catalog::background_searcher_picker_request(canvas, host, at);
    overlay::open_searcher_picker(canvas, host, request);
}

pub(super) fn open_connection_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
) {
    let request = catalog::connection_searcher_picker_request(canvas, host, from, at);
    overlay::open_searcher_picker(canvas, host, request);
}

pub(super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    edge: EdgeId,
    invoked_at: Point,
) {
    super::edge_insert::open_edge_insert_node_picker(canvas, host, window, edge, invoked_at);
}
