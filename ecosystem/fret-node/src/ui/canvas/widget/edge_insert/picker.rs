use super::prelude::*;

pub(in super::super) fn open_edge_insert_node_picker<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    edge: EdgeId,
    invoked_at: Point,
) {
    let request = super::super::searcher_picker::edge_insert_searcher_picker_request(
        edge,
        invoked_at,
        canvas.list_edge_insert_candidates(host, edge),
    );
    if request.candidates.is_empty() {
        canvas.show_toast(
            host,
            window,
            DiagnosticSeverity::Info,
            "no insertable nodes for edge",
        );
        return;
    }

    super::super::searcher_picker::open_searcher_picker_request(canvas, host, request);
}
