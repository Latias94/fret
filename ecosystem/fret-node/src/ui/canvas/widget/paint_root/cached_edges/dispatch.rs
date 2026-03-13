use super::*;

pub(super) fn should_use_tiled_edges_cache(
    edges_cache_tile_size_canvas: f32,
    viewport_w: f32,
    viewport_h: f32,
) -> bool {
    edges_cache_tile_size_canvas.is_finite()
        && (edges_cache_tile_size_canvas < viewport_w || edges_cache_tile_size_canvas < viewport_h)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_root_edges_cached_path_tiled<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    hovered_edge: Option<EdgeId>,
    render_cull_rect: Option<Rect>,
    viewport_rect: Rect,
    zoom: f32,
    view_interacting: bool,
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
    replay_delta: Point,
) {
    canvas.paint_root_edges_cached_path_tiled(
        cx,
        snapshot,
        geom,
        index,
        hovered_edge,
        render_cull_rect,
        viewport_rect,
        zoom,
        view_interacting,
        base_key,
        style_key,
        edges_cache_tile_size_canvas,
        replay_delta,
    );
}

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_root_edges_cached_path_single_rect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &Arc<CanvasGeometry>,
    index: &Arc<CanvasSpatialDerived>,
    hovered_edge: Option<EdgeId>,
    cache_rect: Rect,
    edges_cache_rect: Option<Rect>,
    render_cull_rect: Option<Rect>,
    zoom: f32,
    view_interacting: bool,
    base_key: DerivedBaseKey,
    style_key: u64,
    edges_cache_tile_size_canvas: f32,
    replay_delta: Point,
) {
    canvas.paint_root_edges_cached_path_single_rect(
        cx,
        snapshot,
        geom,
        index,
        hovered_edge,
        cache_rect,
        edges_cache_rect,
        render_cull_rect,
        zoom,
        view_interacting,
        base_key,
        style_key,
        edges_cache_tile_size_canvas,
        replay_delta,
    );
}
