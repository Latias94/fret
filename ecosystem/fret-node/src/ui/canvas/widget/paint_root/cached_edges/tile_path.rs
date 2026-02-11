use super::keys;
use crate::ui::canvas::widget::paint_render_data::RenderData;
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
        let tiles = TileGrid2D::new(edges_cache_tile_size_canvas);
        tiles.tiles_in_rect(edges_rect, &mut self.edges_tiles_scratch);
        tiles.sort_tiles_center_first(viewport_rect, &mut self.edges_tiles_scratch);
        let tiles = self.edges_tiles_scratch.clone();

        if snapshot.interaction.elevate_edges_on_select {
            let edges_base_key =
                keys::edges_tiles_base_key(base_key, style_key, edges_cache_tile_size_canvas);

            let wire_budget_limit = Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
            let marker_budget_limit =
                Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
            let tile_budget_limit =
                Self::EDGE_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
            let mut wire_budget = WorkBudget::new(wire_budget_limit);
            let mut marker_budget = WorkBudget::new(marker_budget_limit);
            let mut tile_budget = WorkBudget::new(tile_budget_limit);

            let mut skipped = false;

            for tile in tiles.iter().copied() {
                let tile_key = tile_cache_key(edges_base_key, tile);
                self.edges_tile_keys_scratch.push(tile_key);

                let tile_origin = tile.origin(edges_cache_tile_size_canvas);
                let tile_rect = Rect::new(
                    tile_origin,
                    Size::new(
                        Px(edges_cache_tile_size_canvas),
                        Px(edges_cache_tile_size_canvas),
                    ),
                );

                if self
                    .edges_scene_cache
                    .try_replay_with(tile_key, cx.scene, replay_delta, |ops| {
                        self.paint_cache.touch_paths_in_scene_ops(ops);
                    })
                {
                    self.edges_build_states.remove(&tile_key);
                    continue;
                }

                if !tile_budget.try_consume(1) {
                    skipped = true;
                    continue;
                }

                let tile_cull_rect = {
                    let margin_screen = self.style.render_cull_margin_px;
                    if margin_screen.is_finite() && margin_screen > 0.0 {
                        inflate_rect(tile_rect, margin_screen / zoom)
                    } else {
                        tile_rect
                    }
                };

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
                    cx.scene.replay_ops_translated(&state.ops, replay_delta);
                    self.paint_cache.touch_paths_in_scene_ops(&state.ops);
                }

                if state.next_edge >= state.edges.len() {
                    self.edges_scene_cache.store_ops(tile_key, state.ops);
                } else {
                    self.edges_build_states.insert(tile_key, state);
                }
            }

            self.edges_build_states
                .retain(|k, _| self.edges_tile_keys_scratch.contains(k));

            if skipped {
                cx.request_redraw();
            }

            self.paint_edge_overlays_selected_hovered(cx, snapshot, geom, zoom);
        } else {
            self.edges_build_states.clear();
            let render_edges: RenderData = self.collect_render_data(
                &*cx.app,
                snapshot,
                Arc::clone(geom),
                Arc::clone(index),
                Some(edges_rect),
                zoom,
                hovered_edge,
                false,
                false,
                true,
            );
            self.paint_edges(cx, snapshot, &render_edges, geom, zoom, view_interacting);
        }

        // --- Edge labels (static, cached tiles) ---
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

            if self.edge_labels_scene_cache.try_replay_with(
                tile_key,
                cx.scene,
                replay_delta,
                |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                },
            ) {
                self.edge_labels_build_states.remove(&tile_key);
                continue;
            }

            if !tile_budget.try_consume(1) {
                skipped_labels = true;
                continue;
            }

            let tile_origin = tile.origin(edges_cache_tile_size_canvas);
            let tile_rect = Rect::new(
                tile_origin,
                Size::new(
                    Px(edges_cache_tile_size_canvas),
                    Px(edges_cache_tile_size_canvas),
                ),
            );

            let tile_cull_rect = {
                let margin_screen = self.style.render_cull_margin_px;
                if margin_screen.is_finite() && margin_screen > 0.0 {
                    inflate_rect(tile_rect, margin_screen / zoom)
                } else {
                    tile_rect
                }
            };

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
                cx.scene.replay_ops_translated(&state.ops, replay_delta);
                self.paint_cache.touch_text_blobs_in_scene_ops(&state.ops);
            }

            if state.next_edge >= state.edges.len() {
                if state.ops.len() == 2 {
                    self.edge_labels_scene_cache.store_ops(tile_key, Vec::new());
                } else {
                    self.edge_labels_scene_cache.store_ops(tile_key, state.ops);
                }
            } else {
                self.edge_labels_build_states.insert(tile_key, state);
            }
        }

        self.edge_labels_build_states
            .retain(|k, _| self.edge_labels_tile_keys_scratch.contains(k));

        if skipped_labels {
            cx.request_redraw();
        }
    }
}
