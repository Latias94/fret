use super::*;

pub(super) fn edge_render_hint<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    edge_id: EdgeId,
) -> EdgeRenderHint {
    EdgePathContext::new(
        &canvas.style,
        &*canvas.presenter,
        canvas.edge_types.as_ref(),
    )
    .edge_render_hint(graph, edge_id)
}

pub(super) fn edge_custom_path<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    edge_id: EdgeId,
    hint: &EdgeRenderHint,
    from: Point,
    to: Point,
    zoom: f32,
) -> Option<crate::ui::edge_types::EdgeCustomPath> {
    EdgePathContext::new(
        &canvas.style,
        &*canvas.presenter,
        canvas.edge_types.as_ref(),
    )
    .edge_custom_path(graph, edge_id, hint, from, to, zoom)
}
