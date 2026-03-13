use super::*;

fn resolve_edge_anchor_color<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    edge_id: EdgeId,
    hint: &EdgeRenderHint,
) -> Color {
    let mut color = canvas.presenter.edge_color(graph, edge_id, &canvas.style);
    if let Some(override_color) = hint.color {
        color = override_color;
    }
    color
}

pub(super) fn resolve_edge_anchor_target_from_geometry<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &PaintCx<'_, H>,
    geom: &CanvasGeometry,
    edge_id: Option<EdgeId>,
) -> Option<EdgeAnchorTarget> {
    let edge_id = edge_id?;
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let edge = graph.edges.get(&edge_id)?;
            let from = geom.port_center(edge.from)?;
            let to = geom.port_center(edge.to)?;
            let hint = EdgePathContext::new(
                &canvas.style,
                &*canvas.presenter,
                canvas.edge_types.as_ref(),
            )
            .edge_render_hint_normalized(graph, edge_id);
            let color = resolve_edge_anchor_color(canvas, graph, edge_id, &hint);
            Some((hint.route, from, to, color))
        })
        .ok()
        .flatten()
}
