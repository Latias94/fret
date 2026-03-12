use super::*;

fn candidate_edge_id<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
) -> Option<EdgeId> {
    canvas
        .interaction
        .focused_edge
        .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
}

fn edge_anchor_target_reconnectable<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    edge_id: EdgeId,
) -> bool {
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let edge = graph.edges.get(&edge_id)?;
            let (allow_source, allow_target) =
                NodeGraphCanvasWith::<M>::edge_reconnectable_flags(edge, &snapshot.interaction);
            Some(allow_source || allow_target)
        })
        .ok()
        .flatten()
        .unwrap_or(false)
}

pub(super) fn resolve_edge_anchor_target_id<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> Option<EdgeId> {
    candidate_edge_id(canvas, snapshot)
        .filter(|edge_id| edge_anchor_target_reconnectable(canvas, cx, snapshot, *edge_id))
}
