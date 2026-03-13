use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_edges_cached_path_tiled<H: UiHost>(
        &mut self,
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
        self.edge_labels_build_state = None;
        self.edges_tiles_scratch.clear();
        self.edges_tile_keys_scratch.clear();

        let edges_rect = render_cull_rect.unwrap_or(viewport_rect);
        let tiles = self.collect_sorted_edge_cache_tiles(
            edges_rect,
            viewport_rect,
            edges_cache_tile_size_canvas,
        );

        if snapshot.interaction.elevate_edges_on_select {
            self.paint_tiled_edges_cache(
                cx,
                snapshot,
                geom,
                index,
                &tiles,
                base_key,
                style_key,
                edges_cache_tile_size_canvas,
                zoom,
                view_interacting,
                replay_delta,
            );
            self.paint_edge_overlays_selected_hovered(cx, snapshot, geom, zoom);
        } else {
            self.paint_root_edges_uncached(
                cx,
                snapshot,
                geom,
                index,
                Some(edges_rect),
                hovered_edge,
                zoom,
                view_interacting,
            );
        }

        self.paint_tiled_edge_labels_cache(
            cx,
            snapshot,
            geom,
            index,
            &tiles,
            base_key,
            style_key,
            edges_cache_tile_size_canvas,
            zoom,
            view_interacting,
            replay_delta,
        );
    }
}
