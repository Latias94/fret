use super::*;

pub(super) fn resolve_cached_edge_anchor_target<H, M, Cx>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &Cx,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
) -> (
    Option<EdgeId>,
    Option<super::anchor_target_adapter::PaintRootCachedEdgeAnchorTarget>,
)
where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: super::anchor_target_adapter::PaintRootCachedEdgeAnchorTargetCx<H>,
{
    super::anchor_target_adapter::resolve_paint_root_cached_edge_anchor_target(
        canvas, cx, snapshot, geom,
    )
}
