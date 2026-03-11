use super::super::*;

pub(super) fn list_background_insert_candidates<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Vec<InsertNodeCandidate> {
    let candidates = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(host, |graph| presenter.list_insertable_nodes(graph))
            .ok()
            .unwrap_or_default()
    };
    super::reroute::prepend_reroute_candidate(candidates)
}

pub(super) fn list_connection_insert_candidates<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    from: PortId,
) -> Vec<InsertNodeCandidate> {
    let candidates = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(host, |graph| {
                presenter.list_insertable_nodes_for_connection(graph, from)
            })
            .ok()
            .unwrap_or_default()
    };
    super::reroute::prepend_reroute_candidate(candidates)
}

pub(super) fn list_edge_insert_candidates<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    edge: EdgeId,
) -> Vec<InsertNodeCandidate> {
    let candidates = {
        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(host, |graph| {
                presenter.list_insertable_nodes_for_edge(graph, edge)
            })
            .ok()
            .unwrap_or_default()
    };
    super::reroute::prepend_reroute_candidate(candidates)
}
