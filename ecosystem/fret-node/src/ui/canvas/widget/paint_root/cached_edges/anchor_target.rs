use super::*;

pub(super) fn resolve_cached_edge_anchor_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
) -> (Option<EdgeId>, Option<(EdgeRouteKind, Point, Point, Color)>) {
    let edge_anchor_target_id = canvas.resolve_edge_anchor_target_id(cx, snapshot);
    let edge_anchor_target =
        canvas.resolve_edge_anchor_target_from_geometry(cx, geom, edge_anchor_target_id);
    (edge_anchor_target_id, edge_anchor_target)
}
