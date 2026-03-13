use crate::ui::canvas::widget::*;

mod anchor_target;
mod build_state;
mod dispatch;
mod edges;
mod fallback;
mod geometry;
mod keys;
mod labels;
mod single_rect;
mod tile_path;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_edges_cached_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        hovered_edge: Option<EdgeId>,
        cache_rect: Rect,
        edges_cache_rect: Option<Rect>,
        render_cull_rect: Option<Rect>,
        viewport_rect: Rect,
        viewport_w: f32,
        viewport_h: f32,
        zoom: f32,
        view_interacting: bool,
        base_key: DerivedBaseKey,
        style_key: u64,
        edges_cache_tile_size_canvas: f32,
    ) -> (Option<EdgeId>, Option<(EdgeRouteKind, Point, Point, Color)>) {
        let replay_delta = Point::new(Px(0.0), Px(0.0));

        let (edge_anchor_target_id, edge_anchor_target) =
            anchor_target::resolve_cached_edge_anchor_target(self, cx, snapshot, geom);

        if crate::ui::canvas::widget::interaction_gate::allow_edges_cache(&self.interaction) {
            if dispatch::should_use_tiled_edges_cache(
                edges_cache_tile_size_canvas,
                viewport_w,
                viewport_h,
            ) {
                dispatch::paint_root_edges_cached_path_tiled(
                    self,
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
            } else {
                dispatch::paint_root_edges_cached_path_single_rect(
                    self,
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
        } else {
            fallback::paint_root_edges_without_static_cache(
                self,
                cx,
                snapshot,
                geom,
                index,
                hovered_edge,
                render_cull_rect,
                zoom,
                view_interacting,
            );
        }

        (edge_anchor_target_id, edge_anchor_target)
    }
}
