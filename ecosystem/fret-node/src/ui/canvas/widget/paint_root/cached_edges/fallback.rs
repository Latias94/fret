use super::*;

fn clear_edge_cache_build_states<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.edges_build_states.clear();
    canvas.edge_labels_build_states.clear();
    canvas.edge_labels_build_state = None;
}

pub(super) fn paint_root_edges_without_static_cache<H, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    hovered_edge: Option<EdgeId>,
    render_cull_rect: Option<Rect>,
    zoom: f32,
    view_interacting: bool,
) where
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
    Cx: super::fallback_adapter::PaintRootCachedEdgeFallbackCx<H>,
{
    clear_edge_cache_build_states(canvas);
    let host = super::fallback_adapter::paint_root_cached_edge_fallback_host(&*cx);
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
    super::fallback_adapter::paint_root_cached_edge_fallback_paint_edges(
        cx,
        canvas,
        snapshot,
        &render_edges,
        geom,
        zoom,
        view_interacting,
    );
}
