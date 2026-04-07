mod overlay;
mod request;

use super::*;

pub(super) fn open_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    at: CanvasPoint,
) {
    let request = request::background_searcher_picker_request(canvas, host, at);
    overlay::open_searcher_picker_request(canvas, host, request);
}

pub(super) fn open_connection_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
    at: CanvasPoint,
) {
    let request = request::connection_searcher_picker_request(canvas, host, from, at);
    overlay::open_searcher_picker_request(canvas, host, request);
}

pub(super) fn open_searcher_picker_request<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    request: request::SearcherPickerRequest,
) {
    overlay::open_searcher_picker_request(canvas, host, request);
}

pub(super) fn conversion_searcher_picker_request(
    from: PortId,
    to: PortId,
    at: CanvasPoint,
    candidates: Vec<InsertNodeCandidate>,
) -> request::SearcherPickerRequest {
    request::conversion_searcher_picker_request(from, to, at, candidates)
}

pub(super) fn edge_insert_searcher_picker_request(
    edge: EdgeId,
    invoked_at: Point,
    candidates: Vec<InsertNodeCandidate>,
) -> request::SearcherPickerRequest {
    request::edge_insert_searcher_picker_request(edge, invoked_at, candidates)
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
