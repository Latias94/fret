use super::geometry::cache_tile_rect;
use super::keys;
use crate::ui::canvas::widget::*;

fn replay_edge_label_ops<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    paint_cache: &mut CanvasPaintCache,
    ops: &[SceneOp],
    replay_delta: Point,
) {
    cx.scene.replay_ops_translated(ops, replay_delta);
    paint_cache.touch_text_blobs_in_scene_ops(ops);
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn store_finished_edge_label_state(&mut self, key: u64, state: EdgeLabelsBuildState) {
        if state.ops.len() == 2 {
            self.edge_labels_scene_cache.store_ops(key, Vec::new());
        } else {
            self.edge_labels_scene_cache.store_ops(key, state.ops);
        }
    }

    pub(super) fn try_replay_cached_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        key: u64,
        replay_delta: Point,
    ) -> bool {
        self.edge_labels_scene_cache
            .try_replay_with(key, cx.scene, replay_delta, |ops| {
                self.paint_cache.touch_text_blobs_in_scene_ops(ops);
            })
    }

    pub(super) fn build_single_rect_edge_labels_cache<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialDerived>,
        labels_key: u64,
        edges_cache_rect: Rect,
        zoom: f32,
        view_interacting: bool,
    ) {
        if self.edge_labels_scene_cache.contains_key(labels_key) {
            self.edge_labels_build_state = None;
            return;
        }

        let mut state = self
            .edge_labels_build_state
            .take()
            .filter(|state| state.key == labels_key)
            .unwrap_or_else(|| {
                self.init_edge_labels_build_state(
                    &*cx.app,
                    snapshot,
                    geom,
                    index,
                    labels_key,
                    edges_cache_rect,
                    edges_cache_rect,
                    zoom,
                )
            });

        let budget_limit = Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut budget = WorkBudget::new(budget_limit);
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

        let mut tmp = fret_core::Scene::default();
        if self.paint_edge_labels_build_state_step(
            &mut tmp,
            &*cx.app,
            cx.services,
            cx.scale_factor,
            zoom,
            bezier_steps,
            &mut state,
            &mut budget,
        ) {
            super::super::redraw_request::request_paint_redraw(cx);
        }

        if state.next_edge >= state.edges.len() {
            self.edge_labels_scene_cache
                .store_ops(labels_key, state.ops.clone());
            self.edge_labels_build_state = None;
        } else {
            self.edge_labels_build_state = Some(state);
        }
    }

    pub(super) fn replay_single_rect_edge_labels<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        labels_key: u64,
        replay_delta: Point,
    ) {
        if self.try_replay_cached_edge_labels(cx, labels_key, replay_delta) {
            return;
        }
        if let Some(state) = self
            .edge_labels_build_state
            .as_ref()
            .filter(|state| state.key == labels_key)
        {
            replay_edge_label_ops(cx, &mut self.paint_cache, &state.ops, replay_delta);
        }
    }

    pub(super) fn paint_tiled_edge_labels_cache<H: UiHost>(
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
        self.edge_labels_tile_keys_scratch.clear();

        let labels_base_key =
            keys::edge_labels_tiles_base_key(base_key, style_key, edges_cache_tile_size_canvas);
        let tile_budget_limit =
            Self::EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
        let label_budget_limit = Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut tile_budget = WorkBudget::new(tile_budget_limit);
        let mut label_budget = WorkBudget::new(label_budget_limit);

        let mut skipped_labels = false;
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

        for tile in tiles.iter().copied() {
            let tile_key = tile_cache_key(labels_base_key, tile);
            self.edge_labels_tile_keys_scratch.push(tile_key);

            if self.try_replay_cached_edge_labels(cx, tile_key, replay_delta) {
                self.edge_labels_build_states.remove(&tile_key);
                continue;
            }

            if !tile_budget.try_consume(1) {
                skipped_labels = true;
                continue;
            }

            let tile_rect = cache_tile_rect(tile, edges_cache_tile_size_canvas);
            let tile_cull_rect = self.cache_tile_cull_rect(tile_rect, zoom);

            let mut state = self
                .edge_labels_build_states
                .remove(&tile_key)
                .unwrap_or_else(|| {
                    self.init_edge_labels_build_state(
                        &*cx.app,
                        snapshot,
                        geom,
                        index,
                        tile_key,
                        tile_rect,
                        tile_cull_rect,
                        zoom,
                    )
                });

            if state.edges.is_empty() {
                self.edge_labels_scene_cache.store_ops(tile_key, Vec::new());
                continue;
            }

            let mut tmp = fret_core::Scene::default();
            if self.paint_edge_labels_build_state_step(
                &mut tmp,
                &*cx.app,
                cx.services,
                cx.scale_factor,
                zoom,
                bezier_steps,
                &mut state,
                &mut label_budget,
            ) {
                skipped_labels = true;
            }

            if state.ops.len() > 2 {
                replay_edge_label_ops(cx, &mut self.paint_cache, &state.ops, replay_delta);
            }

            if state.next_edge >= state.edges.len() {
                self.store_finished_edge_label_state(tile_key, state);
            } else {
                self.edge_labels_build_states.insert(tile_key, state);
            }
        }

        self.edge_labels_build_states
            .retain(|key, _| self.edge_labels_tile_keys_scratch.contains(key));

        super::super::redraw_request::request_paint_redraw_if(cx, skipped_labels);
    }
}
