use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn spatial_index_key(
        snapshot: &ViewSnapshot,
        geom_key: GeometryCacheKey,
        pad_screen_px: f32,
    ) -> SpatialIndexCacheKey {
        let tuning = snapshot.interaction.spatial_index;
        SpatialIndexCacheKey {
            geom: geom_key,
            cell_size_screen_bits: tuning.cell_size_screen_px.to_bits(),
            min_cell_size_screen_bits: tuning.min_cell_size_screen_px.to_bits(),
            edge_aabb_pad_screen_bits: pad_screen_px.to_bits(),
        }
    }

    fn spatial_edge_aabb_pad_screen_px(&self, snapshot: &ViewSnapshot) -> f32 {
        let tuning = snapshot.interaction.spatial_index;
        let mut pad = tuning.edge_aabb_pad_screen_px;
        pad = pad.max(snapshot.interaction.edge_interaction_width);
        pad = pad.max(self.style.wire_width);
        if pad.is_finite() { pad.max(0.0) } else { 0.0 }
    }

    fn spatial_index_params(&self, snapshot: &ViewSnapshot) -> SpatialIndexParams {
        let zoom = snapshot.zoom;
        let pad_screen_px = self.spatial_edge_aabb_pad_screen_px(snapshot);
        let z = zoom.max(1.0e-6);
        let tuning = snapshot.interaction.spatial_index;
        let cell_size_canvas = (tuning.cell_size_screen_px / z)
            .max(tuning.min_cell_size_screen_px / z)
            .max(1.0);
        let max_hit_pad_canvas = (pad_screen_px / z).max(0.0);
        SpatialIndexParams {
            pad_screen_px,
            cell_size_canvas,
            max_hit_pad_canvas,
        }
    }

    fn patch_spatial_index_with_custom_edge_paths(
        graph: &Graph,
        geom: &CanvasGeometry,
        zoom: f32,
        edge_ctx: EdgePathContext<'_>,
        max_hit_pad_canvas: f32,
        index: &mut CanvasSpatialIndex,
    ) {
        if !edge_ctx.has_custom_paths() {
            return;
        }

        let z = zoom.max(1.0e-6);
        let pin_pad = (edge_ctx.style.pin_radius.max(0.0) / z).max(0.0);

        for (&edge_id, edge) in &graph.edges {
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };
            let Some(bounds) = Self::custom_edge_path_bounds_rect(
                graph,
                zoom,
                edge_ctx,
                edge_id,
                from,
                to,
                max_hit_pad_canvas,
                pin_pad,
            ) else {
                continue;
            };
            index.update_edge_rect(edge_id, bounds);
        }
    }

    fn custom_edge_path_bounds_rect(
        graph: &Graph,
        zoom: f32,
        edge_ctx: EdgePathContext<'_>,
        edge_id: EdgeId,
        from: Point,
        to: Point,
        max_hit_pad_canvas: f32,
        pin_pad_canvas: f32,
    ) -> Option<Rect> {
        let z = zoom.max(1.0e-6);
        let hint = edge_ctx.edge_render_hint_normalized(graph, edge_id);
        let marker_pad_canvas = hint
            .start_marker
            .as_ref()
            .map(|m| (m.size.max(0.0) / z).max(0.0))
            .unwrap_or(0.0)
            .max(
                hint.end_marker
                    .as_ref()
                    .map(|m| (m.size.max(0.0) / z).max(0.0))
                    .unwrap_or(0.0),
            );
        let pad = max_hit_pad_canvas
            .max(pin_pad_canvas)
            .max(marker_pad_canvas);

        let custom = edge_ctx.edge_custom_path(graph, edge_id, &hint, from, to, zoom)?;
        let bounds = path_bounds_rect(&custom.commands)?;
        Some(inflate_rect(bounds, pad))
    }

    pub(super) fn ensure_spatial_index_cache<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom_key: GeometryCacheKey,
        geom: &Arc<CanvasGeometry>,
    ) -> Arc<CanvasSpatialIndex> {
        let zoom = snapshot.zoom;
        let params = self.spatial_index_params(snapshot);
        let index_key = Self::spatial_index_key(snapshot, geom_key, params.pad_screen_px);
        if self.geometry.ensure_index_key(index_key) {
            let style = self.style.clone();
            let edge_types = self.edge_types.as_ref();
            let presenter: &dyn NodeGraphPresenter = &*self.presenter;
            let edge_ctx = EdgePathContext::new(&style, presenter, edge_types);

            let index = self
                .graph
                .read_ref(host, |graph| {
                    let mut index = CanvasSpatialIndex::build(
                        graph,
                        geom,
                        zoom,
                        params.max_hit_pad_canvas,
                        params.cell_size_canvas,
                    );

                    // Stage 2 `edgeTypes`: custom edge paths may exceed the default conservative
                    // wire AABB, so patch the index with a custom bounds rect when available.
                    if edge_ctx.has_custom_paths() {
                        Self::patch_spatial_index_with_custom_edge_paths(
                            graph,
                            geom,
                            zoom,
                            edge_ctx,
                            params.max_hit_pad_canvas,
                            &mut index,
                        );
                    }

                    index
                })
                .ok()
                .unwrap_or_else(CanvasSpatialIndex::empty);

            self.geometry.index = Arc::new(index);
        }

        self.geometry.index.clone()
    }
}

#[derive(Debug, Clone, Copy)]
struct SpatialIndexParams {
    pad_screen_px: f32,
    cell_size_canvas: f32,
    max_hit_pad_canvas: f32,
}
