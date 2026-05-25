use super::*;

pub(super) fn paint_root_edges_uncached<H, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    render_cull_rect: Option<Rect>,
    hovered_edge: Option<EdgeId>,
    zoom: f32,
    view_interacting: bool,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: super::super::fallback_adapter::PaintRootCachedEdgeFallbackCx<H>,
{
    canvas.edges_build_states.clear();
    let host = super::super::fallback_adapter::paint_root_cached_edge_fallback_host(&*cx);
    let render_edges = canvas.collect_render_data(
        host,
        snapshot,
        Arc::clone(geom),
        Arc::clone(index),
        render_cull_rect,
        zoom,
        hovered_edge,
        false,
        false,
        true,
    );
    super::super::fallback_adapter::paint_root_cached_edge_fallback_paint_edges(
        cx,
        canvas,
        snapshot,
        &render_edges,
        geom,
        zoom,
        view_interacting,
    );
}
