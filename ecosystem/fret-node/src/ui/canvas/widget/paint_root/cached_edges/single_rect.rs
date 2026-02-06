use super::keys;
use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root_edges_cached_path_single_rect<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
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
            let edges_hit =
                self.edges_scene_cache
                    .try_replay_with(edges_key, cx.scene, replay_delta, |ops| {
                        self.paint_cache.touch_paths_in_scene_ops(ops);
                    });
            if edges_hit {
                self.edges_build_states.remove(&edges_key);
            } else {
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

                let wire_budget_limit =
                    Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
                let marker_budget_limit =
                    Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
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
                    cx.request_redraw();
                }

                if state.edges.is_empty() {
                    self.edges_scene_cache.store_ops(edges_key, Vec::new());
                } else if state.ops.len() > 2 {
                    cx.scene.replay_ops_translated(&state.ops, replay_delta);
                    if state.next_edge >= state.edges.len() {
                        self.paint_cache.touch_paths_in_scene_ops(&state.ops);
                        self.edges_scene_cache.store_ops(edges_key, state.ops);
                    } else {
                        self.paint_cache.touch_paths_in_scene_ops(&state.ops);
                        self.edges_build_states.insert(edges_key, state);
                    }
                } else {
                    self.paint_cache.touch_paths_in_scene_ops(&state.ops);
                    self.edges_build_states.insert(edges_key, state);
                }
            }
        } else {
            self.edges_build_states.remove(&edges_key);
        }

        if self.edge_labels_scene_cache.contains_key(labels_key) {
            self.edge_labels_build_state = None;
        } else {
            let mut state = self
                .edge_labels_build_state
                .take()
                .filter(|s| s.key == labels_key)
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
                cx.request_redraw();
            }

            if state.next_edge >= state.edges.len() {
                self.edge_labels_scene_cache
                    .store_ops(labels_key, state.ops.clone());
                self.edge_labels_build_state = None;
            } else {
                self.edge_labels_build_state = Some(state);
            }
        }

        if snapshot.interaction.elevate_edges_on_select {
            self.paint_edge_overlays_selected_hovered(cx, snapshot, geom, zoom);

            // Labels must remain on top of selected/hovered overlays.
            if self.edge_labels_scene_cache.try_replay_with(
                labels_key,
                cx.scene,
                replay_delta,
                |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                },
            ) {
            } else if let Some(state) = self
                .edge_labels_build_state
                .as_ref()
                .filter(|s| s.key == labels_key)
            {
                cx.scene.replay_ops_translated(&state.ops, replay_delta);
                self.paint_cache.touch_text_blobs_in_scene_ops(&state.ops);
            }
        } else {
            // Labels may be present in the cache without being replayed (hit path).
            if self.edge_labels_scene_cache.try_replay_with(
                labels_key,
                cx.scene,
                replay_delta,
                |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                },
            ) {
            } else if let Some(state) = self
                .edge_labels_build_state
                .as_ref()
                .filter(|s| s.key == labels_key)
            {
                cx.scene.replay_ops_translated(&state.ops, replay_delta);
                self.paint_cache.touch_text_blobs_in_scene_ops(&state.ops);
            }
        }
    }
}
