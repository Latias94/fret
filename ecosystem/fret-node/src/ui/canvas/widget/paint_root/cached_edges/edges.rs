use super::geometry::cache_tile_rect;
use super::keys;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn replay_cached_edge_build_state<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        state: &EdgesBuildState,
        replay_delta: Point,
    ) {
        cx.scene.replay_ops_translated(&state.ops, replay_delta);
        self.paint_cache.touch_paths_in_scene_ops(&state.ops);
    }

    fn store_finished_edge_build_state(&mut self, key: u64, state: EdgesBuildState) {
        self.edges_scene_cache.store_ops(key, state.ops);
    }

    pub(super) fn paint_root_edges_uncached<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        render_cull_rect: Option<Rect>,
        hovered_edge: Option<EdgeId>,
        zoom: f32,
        view_interacting: bool,
    ) {
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

    pub(super) fn try_replay_cached_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        key: u64,
        replay_delta: Point,
    ) -> bool {
        self.edges_scene_cache
            .try_replay_with(key, cx.scene, replay_delta, |ops| {
                self.paint_cache.touch_paths_in_scene_ops(ops);
            })
    }

    pub(super) fn build_single_rect_edges_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        edges_key: u64,
        edges_cache_rect: Rect,
        zoom: f32,
        view_interacting: bool,
        replay_delta: Point,
    ) {
        if self.try_replay_cached_edges(cx, edges_key, replay_delta) {
            self.edges_build_states.remove(&edges_key);
            return;
        }

        let mut state = self
            .edges_build_states
            .remove(&edges_key)
            .unwrap_or_else(|| {
                self.init_edges_build_state(
                    &*cx.app,
                    snapshot,
                    geom,
                    index,
                    edges_cache_rect,
                    edges_cache_rect,
                    zoom,
                )
            });

        let wire_budget_limit = Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let marker_budget_limit = Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut wire_budget = WorkBudget::new(wire_budget_limit);
        let mut marker_budget = WorkBudget::new(marker_budget_limit);

        let mut tmp = fret_core::Scene::default();
        if self.paint_edges_build_state_step(
            &mut tmp,
            &*cx.app,
            cx.services,
            zoom,
            cx.scale_factor,
            &mut state,
            &mut wire_budget,
            &mut marker_budget,
        ) {
            super::super::redraw_request::request_paint_redraw(cx);
        }

        if state.edges.is_empty() {
            self.edges_scene_cache.store_ops(edges_key, Vec::new());
        } else if state.ops.len() > 2 {
            self.replay_cached_edge_build_state(cx, &state, replay_delta);
            if state.next_edge >= state.edges.len() {
                self.store_finished_edge_build_state(edges_key, state);
            } else {
                self.edges_build_states.insert(edges_key, state);
            }
        } else {
            self.paint_cache.touch_paths_in_scene_ops(&state.ops);
            self.edges_build_states.insert(edges_key, state);
        }
    }

    pub(super) fn paint_tiled_edges_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        tiles: &[TileCoord],
        base_key: DerivedBaseKey,
        style_key: u64,
        edges_cache_tile_size_canvas: f32,
        zoom: f32,
        view_interacting: bool,
        replay_delta: Point,
    ) {
        let edges_base_key =
            keys::edges_tiles_base_key(base_key, style_key, edges_cache_tile_size_canvas);

        let wire_budget_limit = Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let marker_budget_limit = Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let tile_budget_limit =
            Self::EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
        let mut wire_budget = WorkBudget::new(wire_budget_limit);
        let mut marker_budget = WorkBudget::new(marker_budget_limit);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);

        let mut skipped = false;

        for tile in tiles.iter().copied() {
            let tile_key = tile_cache_key(edges_base_key, tile);
            self.edges_tile_keys_scratch.push(tile_key);

            if self.try_replay_cached_edges(cx, tile_key, replay_delta) {
                self.edges_build_states.remove(&tile_key);
                continue;
            }

            if !tile_budget.try_consume(1) {
                skipped = true;
                continue;
            }

            let tile_rect = cache_tile_rect(tile, edges_cache_tile_size_canvas);
            let tile_cull_rect = self.cache_tile_cull_rect(tile_rect, zoom);

            let mut state = self
                .edges_build_states
                .remove(&tile_key)
                .unwrap_or_else(|| {
                    self.init_edges_build_state(
                        &*cx.app,
                        snapshot,
                        geom,
                        index,
                        tile_rect,
                        tile_cull_rect,
                        zoom,
                    )
                });

            let mut tmp = fret_core::Scene::default();
            if self.paint_edges_build_state_step(
                &mut tmp,
                &*cx.app,
                cx.services,
                zoom,
                cx.scale_factor,
                &mut state,
                &mut wire_budget,
                &mut marker_budget,
            ) {
                skipped = true;
            }

            if state.edges.is_empty() {
                self.edges_scene_cache.store_ops(tile_key, Vec::new());
                continue;
            }

            if state.ops.len() > 2 {
                self.replay_cached_edge_build_state(cx, &state, replay_delta);
            }

            if state.next_edge >= state.edges.len() {
                self.store_finished_edge_build_state(tile_key, state);
            } else {
                self.edges_build_states.insert(tile_key, state);
            }
        }

        self.edges_build_states
            .retain(|key, _| self.edges_tile_keys_scratch.contains(key));

        super::super::redraw_request::request_paint_redraw_if(cx, skipped);
    }
}
