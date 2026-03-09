use super::keys;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_edges_cached_path_single_rect<H: UiHost>(
        &mut self,
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
        let edges_cache_rect = edges_cache_rect.unwrap_or(cache_rect);

        let edges_key = keys::edges_single_rect_key(
            base_key,
            style_key,
            edges_cache_tile_size_canvas,
            edges_cache_rect,
        );

        if !snapshot.interaction.elevate_edges_on_select {
            self.edges_build_states.clear();
            let render_edges: RenderData = self.collect_render_data(
                &*cx.app,
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
            self.paint_edges(cx, snapshot, &render_edges, geom, zoom, view_interacting);
        }

        let labels_key = keys::edge_labels_single_rect_key(
            base_key,
            style_key,
            edges_cache_tile_size_canvas,
            edges_cache_rect,
        );

        if snapshot.interaction.elevate_edges_on_select {
            self.build_single_rect_edges_cache(
                cx,
                snapshot,
                geom,
                index,
                edges_key,
                edges_cache_rect,
                zoom,
                view_interacting,
                replay_delta,
            );
        } else {
            self.edges_build_states.remove(&edges_key);
        }

        self.build_single_rect_edge_labels_cache(
            cx,
            snapshot,
            geom,
            index,
            labels_key,
            edges_cache_rect,
            zoom,
            view_interacting,
        );

        if snapshot.interaction.elevate_edges_on_select {
            self.paint_edge_overlays_selected_hovered(cx, snapshot, geom, zoom);
        }
        self.replay_single_rect_edge_labels(cx, labels_key, replay_delta);
    }
}
