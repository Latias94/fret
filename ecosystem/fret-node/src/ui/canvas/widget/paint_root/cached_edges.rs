use super::super::super::paint_render_data::RenderData;
use super::super::super::*;

fn extend_clip_stack_ops(ops: &mut Vec<SceneOp>, tmp: &[SceneOp]) {
    if tmp.is_empty() {
        return;
    }

    match ops.pop() {
        Some(SceneOp::PopClip) => {
            ops.extend_from_slice(tmp);
            ops.push(SceneOp::PopClip);
        }
        Some(other) => {
            ops.push(other);
            ops.extend_from_slice(tmp);
        }
        None => {
            ops.extend_from_slice(tmp);
        }
    }

    if !matches!(ops.last(), Some(SceneOp::PopClip)) {
        ops.push(SceneOp::PopClip);
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn init_edges_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgesBuildState {
        let render_edges: RenderData = self.collect_render_data(
            host,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
            Some(cull_rect),
            zoom,
            None,
            false,
            false,
            true,
        );
        EdgesBuildState {
            ops: vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip],
            edges: render_edges.edges,
            next_edge: 0,
        }
    }

    fn init_edge_labels_build_state<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
        key: u64,
        clip_rect: Rect,
        cull_rect: Rect,
        zoom: f32,
    ) -> EdgeLabelsBuildState {
        let render_edges: RenderData = self.collect_render_data(
            host,
            snapshot,
            Arc::clone(geom),
            Arc::clone(index),
            Some(cull_rect),
            zoom,
            None,
            false,
            false,
            true,
        );
        EdgeLabelsBuildState {
            key,
            ops: vec![SceneOp::PushClipRect { rect: clip_rect }, SceneOp::PopClip],
            edges: render_edges.edges,
            next_edge: 0,
        }
    }

    fn paint_edges_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        state: &mut EdgesBuildState,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
    ) -> bool {
        let (next_edge, skipped) = self.paint_edges_cached_budgeted(
            tmp,
            host,
            services,
            &state.edges,
            zoom,
            scale_factor,
            state.next_edge,
            wire_budget,
            marker_budget,
        );
        state.next_edge = next_edge;

        extend_clip_stack_ops(&mut state.ops, tmp.ops());
        skipped || state.next_edge < state.edges.len()
    }

    fn paint_edge_labels_build_state_step<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        zoom: f32,
        bezier_steps: usize,
        state: &mut EdgeLabelsBuildState,
        budget: &mut WorkBudget,
    ) -> bool {
        let (next_edge, skipped) = self.paint_edge_labels_static_budgeted_cached(
            tmp,
            host,
            services,
            scale_factor,
            &state.edges,
            bezier_steps,
            zoom,
            state.next_edge,
            budget,
        );
        state.next_edge = next_edge;

        extend_clip_stack_ops(&mut state.ops, tmp.ops());
        skipped || state.next_edge < state.edges.len()
    }

    pub(super) fn paint_root_edges_cached_path<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &Arc<CanvasGeometry>,
        index: &Arc<CanvasSpatialIndex>,
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

        // --- Edges (static + overlays) ---
        let edges_cache_allowed = self.interaction.pending_wire_drag.is_none()
            && self.interaction.wire_drag.is_none()
            && self.interaction.suspended_wire_drag.is_none()
            && self.interaction.pending_edge_insert_drag.is_none()
            && self.interaction.edge_insert_drag.is_none()
            && self.interaction.edge_drag.is_none()
            && self.interaction.pending_insert_node_drag.is_none()
            && self.interaction.insert_node_drag_preview.is_none();

        let edge_anchor_target_id = self
            .interaction
            .focused_edge
            .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
            .filter(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(edge_id)?;
                        let (allow_source, allow_target) =
                            Self::edge_reconnectable_flags(edge, &snapshot.interaction);
                        Some(allow_source || allow_target)
                    })
                    .ok()
                    .flatten()
                    .unwrap_or(false)
            });
        let edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)> =
            edge_anchor_target_id.and_then(|edge_id| {
                self.graph
                    .read_ref(cx.app, |g| {
                        let edge = g.edges.get(&edge_id)?;
                        let from = geom.port_center(edge.from)?;
                        let to = geom.port_center(edge.to)?;
                        let hint = EdgePathContext::new(
                            &self.style,
                            &*self.presenter,
                            self.edge_types.as_ref(),
                        )
                        .edge_render_hint_normalized(g, edge_id);
                        let mut color = self.presenter.edge_color(g, edge_id, &self.style);
                        if let Some(override_color) = hint.color {
                            color = override_color;
                        }
                        Some((hint.route, from, to, color))
                    })
                    .ok()
                    .flatten()
            });

        if edges_cache_allowed {
            if edges_cache_tile_size_canvas.is_finite()
                && (edges_cache_tile_size_canvas < viewport_w
                    || edges_cache_tile_size_canvas < viewport_h)
            {
                self.edge_labels_build_state = None;
                self.edges_tiles_scratch.clear();
                self.edges_tile_keys_scratch.clear();

                let edges_rect = render_cull_rect.unwrap_or(viewport_rect);
                let tiles = TileGrid2D::new(edges_cache_tile_size_canvas);
                tiles.tiles_in_rect(edges_rect, &mut self.edges_tiles_scratch);
                tiles.sort_tiles_center_first(viewport_rect, &mut self.edges_tiles_scratch);
                let tiles = self.edges_tiles_scratch.clone();

                if snapshot.interaction.elevate_edges_on_select {
                    let edges_base_key = {
                        let mut b =
                            TileCacheKeyBuilder::new("fret-node.canvas.static_edges.tile.v1");
                        b.add_u64(base_key.graph_rev);
                        b.add_u32(base_key.zoom_bits);
                        b.add_u32(base_key.node_origin_x_bits);
                        b.add_u32(base_key.node_origin_y_bits);
                        b.add_u64(base_key.draw_order_hash);
                        b.add_u64(base_key.presenter_rev);
                        b.add_u64(base_key.edge_types_rev);
                        b.add_u64(style_key);
                        b.add_f32_bits(edges_cache_tile_size_canvas);
                        b.finish()
                    };

                    let wire_budget_limit =
                        Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
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

                        if self.edges_scene_cache.try_replay_with(
                            tile_key,
                            cx.scene,
                            replay_delta,
                            |ops| {
                                self.paint_cache.touch_paths_in_scene_ops(ops);
                            },
                        ) {
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

                        let mut state =
                            self.edges_build_states
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

                let labels_base_key = {
                    let mut b =
                        TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.tile.v1");
                    b.add_u64(base_key.graph_rev);
                    b.add_u32(base_key.zoom_bits);
                    b.add_u32(base_key.node_origin_x_bits);
                    b.add_u32(base_key.node_origin_y_bits);
                    b.add_u64(base_key.draw_order_hash);
                    b.add_u64(base_key.presenter_rev);
                    b.add_u64(base_key.edge_types_rev);
                    b.add_u64(style_key);
                    b.add_f32_bits(edges_cache_tile_size_canvas);
                    b.finish()
                };

                let tile_budget_limit =
                    Self::EDGE_LABEL_TILE_BUILD_BUDGET_TILES_PER_FRAME.select(view_interacting);
                let label_budget_limit =
                    Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
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
            } else {
                let edges_cache_rect = edges_cache_rect.unwrap_or(cache_rect);

                let edges_key = {
                    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edges.v1");
                    b.add_u64(base_key.graph_rev);
                    b.add_u32(base_key.zoom_bits);
                    b.add_u32(base_key.node_origin_x_bits);
                    b.add_u32(base_key.node_origin_y_bits);
                    b.add_u64(base_key.draw_order_hash);
                    b.add_u64(base_key.presenter_rev);
                    b.add_u64(base_key.edge_types_rev);
                    b.add_u64(style_key);
                    b.add_f32_bits(edges_cache_tile_size_canvas);
                    b.add_u32(edges_cache_rect.origin.x.0.to_bits());
                    b.add_u32(edges_cache_rect.origin.y.0.to_bits());
                    b.finish()
                };

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

                let labels_key = {
                    let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.v1");
                    b.add_u64(base_key.graph_rev);
                    b.add_u32(base_key.zoom_bits);
                    b.add_u32(base_key.node_origin_x_bits);
                    b.add_u32(base_key.node_origin_y_bits);
                    b.add_u64(base_key.draw_order_hash);
                    b.add_u64(base_key.presenter_rev);
                    b.add_u64(base_key.edge_types_rev);
                    b.add_u64(style_key);
                    b.add_f32_bits(edges_cache_tile_size_canvas);
                    b.add_u32(edges_cache_rect.origin.x.0.to_bits());
                    b.add_u32(edges_cache_rect.origin.y.0.to_bits());
                    b.finish()
                };

                if snapshot.interaction.elevate_edges_on_select {
                    let edges_hit = self.edges_scene_cache.try_replay_with(
                        edges_key,
                        cx.scene,
                        replay_delta,
                        |ops| {
                            self.paint_cache.touch_paths_in_scene_ops(ops);
                        },
                    );
                    if edges_hit {
                        self.edges_build_states.remove(&edges_key);
                    } else {
                        let mut state =
                            self.edges_build_states
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

                        if state.next_edge >= state.edges.len() {
                            self.edges_scene_cache
                                .store_ops(edges_key, state.ops.clone());
                            let _ = self.edges_scene_cache.try_replay_with(
                                edges_key,
                                cx.scene,
                                replay_delta,
                                |ops| {
                                    self.paint_cache.touch_paths_in_scene_ops(ops);
                                },
                            );
                        } else {
                            cx.scene.replay_ops_translated(&state.ops, replay_delta);
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

                    let budget_limit =
                        Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
                    let mut budget = WorkBudget::new(budget_limit);
                    let bezier_steps =
                        usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));

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
        } else {
            self.edges_build_states.clear();
            self.edge_labels_build_states.clear();
            self.edge_labels_build_state = None;
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

        (edge_anchor_target_id, edge_anchor_target)
    }
}
