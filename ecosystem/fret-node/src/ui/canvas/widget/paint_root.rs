use super::paint_render_data::RenderData;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    const STATIC_NODES_TILE_MUL: f32 = 2.0;
    const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
    const STATIC_EDGES_TILE_SIZE_SCREEN_PX: u32 = 2048;

    fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
        let target = value.ceil().max(1.0) as u32;
        let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
        pow2.max(min)
    }

    pub(super) fn paint_root<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        let view_interacting = self.view_interacting();

        self.paint_cache.begin_frame();
        self.groups_scene_cache.begin_frame();
        self.nodes_scene_cache.begin_frame();
        self.edges_scene_cache.begin_frame();
        self.edge_labels_scene_cache.begin_frame();
        if let Some(window) = cx.window {
            let (entries, stats) = self.paint_cache.diagnostics_path_cache_snapshot();
            let frame_id = cx.app.frame_id().0;
            let key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-node.canvas.paths",
            };
            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_path_cache(key, frame_id, entries, stats);
                });
        }

        let zoom = snapshot.zoom;
        let pan = snapshot.pan;

        let viewport = CanvasViewport2D::new(
            cx.bounds,
            PanZoom2D {
                pan: Point::new(Px(pan.x), Px(pan.y)),
                zoom,
            },
        );
        let viewport_rect = viewport.visible_canvas_rect();
        let viewport_w = viewport_rect.size.width.0;
        let viewport_h = viewport_rect.size.height.0;
        let viewport_origin_x = viewport_rect.origin.x.0;
        let viewport_origin_y = viewport_rect.origin.y.0;
        let only_render_visible_elements = snapshot.interaction.only_render_visible_elements;
        let render_cull_rect = if !only_render_visible_elements {
            None
        } else {
            let margin_screen = self.style.render_cull_margin_px;
            if !margin_screen.is_finite()
                || margin_screen <= 0.0
                || !viewport_w.is_finite()
                || !viewport_h.is_finite()
                || viewport_w <= 0.0
                || viewport_h <= 0.0
            {
                None
            } else {
                let margin = margin_screen / zoom;
                Some(inflate_rect(viewport_rect, margin))
            }
        };

        cx.scene.push(SceneOp::PushClipRect {
            rect: viewport_rect,
        });

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(0),
            rect: viewport_rect,
            background: self.style.background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(0.0)),
        });

        self.paint_grid(cx, viewport_rect, render_cull_rect, zoom, view_interacting);

        let edge_insert_target = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|d| d.edge)
            .or_else(|| {
                self.interaction
                    .pending_edge_insert_drag
                    .as_ref()
                    .map(|d| d.edge)
            });
        let insert_node_drag_edge = self
            .interaction
            .insert_node_drag_preview
            .as_ref()
            .and_then(|p| p.edge);
        let hovered_edge = edge_insert_target
            .or(insert_node_drag_edge)
            .or(self.interaction.hover_edge);

        let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
        self.update_measured_output_store(snapshot.zoom, &geom);
        self.update_internals_store(&*cx.app, &snapshot, cx.bounds, &geom);

        let can_use_static_scene_cache = self.geometry.drag_preview.is_none()
            && only_render_visible_elements
            && zoom.is_finite()
            && zoom > 1.0e-6
            && cx.bounds.size.width.0.is_finite()
            && cx.bounds.size.height.0.is_finite();

        let viewport_max_screen_px = cx.bounds.size.width.0.max(cx.bounds.size.height.0);
        let nodes_tile_size_screen_px = Self::next_power_of_two_at_least(
            Self::STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN,
            viewport_max_screen_px * Self::STATIC_NODES_TILE_MUL,
        );

        let nodes_cache_tile_size_canvas = (nodes_tile_size_screen_px as f32 / zoom).max(1.0);
        let edges_cache_tile_size_canvas =
            (Self::STATIC_EDGES_TILE_SIZE_SCREEN_PX as f32 / zoom).max(1.0);

        let nodes_cache_rect: Option<Rect> = if can_use_static_scene_cache
            && nodes_cache_tile_size_canvas >= viewport_w
            && nodes_cache_tile_size_canvas >= viewport_h
        {
            let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
            let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
            if center_x.is_finite() && center_y.is_finite() {
                let tile_x = (center_x / nodes_cache_tile_size_canvas).floor() as i32;
                let tile_y = (center_y / nodes_cache_tile_size_canvas).floor() as i32;
                let origin = Point::new(
                    Px(tile_x as f32 * nodes_cache_tile_size_canvas),
                    Px(tile_y as f32 * nodes_cache_tile_size_canvas),
                );
                Some(Rect::new(
                    origin,
                    Size::new(
                        Px(nodes_cache_tile_size_canvas),
                        Px(nodes_cache_tile_size_canvas),
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        };

        let edges_cache_rect: Option<Rect> = if can_use_static_scene_cache
            && edges_cache_tile_size_canvas >= viewport_w
            && edges_cache_tile_size_canvas >= viewport_h
        {
            let center_x = viewport_rect.origin.x.0 + 0.5 * viewport_rect.size.width.0;
            let center_y = viewport_rect.origin.y.0 + 0.5 * viewport_rect.size.height.0;
            if center_x.is_finite() && center_y.is_finite() {
                let tile_x = (center_x / edges_cache_tile_size_canvas).floor() as i32;
                let tile_y = (center_y / edges_cache_tile_size_canvas).floor() as i32;
                let origin = Point::new(
                    Px(tile_x as f32 * edges_cache_tile_size_canvas),
                    Px(tile_y as f32 * edges_cache_tile_size_canvas),
                );
                Some(Rect::new(
                    origin,
                    Size::new(
                        Px(edges_cache_tile_size_canvas),
                        Px(edges_cache_tile_size_canvas),
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        };

        let style_key: u64 = {
            let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_scene_style.v1");
            b.add_u32(self.style.group_background.r.to_bits());
            b.add_u32(self.style.group_background.g.to_bits());
            b.add_u32(self.style.group_background.b.to_bits());
            b.add_u32(self.style.group_background.a.to_bits());
            b.add_u32(self.style.group_border.r.to_bits());
            b.add_u32(self.style.group_border.g.to_bits());
            b.add_u32(self.style.group_border.b.to_bits());
            b.add_u32(self.style.group_border.a.to_bits());
            b.add_u32(self.style.node_background.r.to_bits());
            b.add_u32(self.style.node_background.g.to_bits());
            b.add_u32(self.style.node_background.b.to_bits());
            b.add_u32(self.style.node_background.a.to_bits());
            b.add_u32(self.style.node_border.r.to_bits());
            b.add_u32(self.style.node_border.g.to_bits());
            b.add_u32(self.style.node_border.b.to_bits());
            b.add_u32(self.style.node_border.a.to_bits());
            b.add_u32(self.style.wire_color_data.r.to_bits());
            b.add_u32(self.style.wire_color_data.g.to_bits());
            b.add_u32(self.style.wire_color_data.b.to_bits());
            b.add_u32(self.style.wire_color_data.a.to_bits());
            b.add_u32(self.style.wire_color_exec.r.to_bits());
            b.add_u32(self.style.wire_color_exec.g.to_bits());
            b.add_u32(self.style.wire_color_exec.b.to_bits());
            b.add_u32(self.style.wire_color_exec.a.to_bits());
            b.add_u32(self.style.wire_width.to_bits());
            b.add_u32(self.style.wire_width_selected_mul.to_bits());
            b.add_u32(self.style.wire_width_hover_mul.to_bits());
            b.add_u32(self.style.context_menu_background.r.to_bits());
            b.add_u32(self.style.context_menu_background.g.to_bits());
            b.add_u32(self.style.context_menu_background.b.to_bits());
            b.add_u32(self.style.context_menu_background.a.to_bits());
            b.add_u32(self.style.context_menu_border.r.to_bits());
            b.add_u32(self.style.context_menu_border.g.to_bits());
            b.add_u32(self.style.context_menu_border.b.to_bits());
            b.add_u32(self.style.context_menu_border.a.to_bits());
            b.add_u32(self.style.context_menu_text.r.to_bits());
            b.add_u32(self.style.context_menu_text.g.to_bits());
            b.add_u32(self.style.context_menu_text.b.to_bits());
            b.add_u32(self.style.context_menu_text.a.to_bits());
            b.add_f32_bits(self.style.node_padding);
            b.add_f32_bits(self.style.node_header_height);
            b.add_f32_bits(self.style.pin_row_height);
            b.add_f32_bits(self.style.pin_radius);
            b.add_u32(self.style.context_menu_text_style.size.0.to_bits());
            b.add_u32(u32::from(self.style.context_menu_text_style.weight.0));
            b.add_u32(cx.scale_factor.to_bits());
            b.finish()
        };

        let node_origin = snapshot.interaction.node_origin.normalized();
        let geom_key = self.geometry.key.unwrap_or(GeometryCacheKey {
            graph_rev: self.graph.revision(&*cx.app).unwrap_or(0),
            zoom_bits: zoom.to_bits(),
            node_origin_x_bits: node_origin.x.to_bits(),
            node_origin_y_bits: node_origin.y.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev: self.presenter.geometry_revision(),
            edge_types_rev: self.edge_types.as_ref().map(|t| t.revision()).unwrap_or(0),
        });

        if let Some(cache_rect) = nodes_cache_rect {
            // --- Groups (static, cached) ---
            let groups_key = {
                let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_groups.v1");
                b.add_u64(geom_key.graph_rev);
                b.add_u32(geom_key.zoom_bits);
                b.add_u32(geom_key.node_origin_x_bits);
                b.add_u32(geom_key.node_origin_y_bits);
                b.add_u64(geom_key.draw_order_hash);
                b.add_u64(geom_key.presenter_rev);
                b.add_u64(geom_key.edge_types_rev);
                b.add_u64(style_key);
                b.add_f32_bits(nodes_cache_tile_size_canvas);
                b.add_u32(cache_rect.origin.x.0.to_bits());
                b.add_u32(cache_rect.origin.y.0.to_bits());
                b.finish()
            };

            let replay_delta = Point::new(Px(0.0), Px(0.0));
            let groups_hit = self.groups_scene_cache.try_replay_with(
                groups_key,
                cx.scene,
                replay_delta,
                |ops| {
                    self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                },
            );
            if !groups_hit {
                let render_groups: RenderData = self.collect_render_data(
                    &*cx.app,
                    &snapshot,
                    geom.clone(),
                    index.clone(),
                    Some(cache_rect),
                    zoom,
                    None,
                    true,
                    false,
                    false,
                );

                let mut tmp = fret_core::Scene::default();
                tmp.push(SceneOp::PushClipRect { rect: cache_rect });
                self.paint_groups_static(
                    &mut tmp,
                    cx.services,
                    cx.scale_factor,
                    &render_groups.groups,
                    zoom,
                );
                tmp.push(SceneOp::PopClip);
                self.groups_scene_cache
                    .store_ops(groups_key, tmp.ops().to_vec());
                let _ = self.groups_scene_cache.try_replay_with(
                    groups_key,
                    cx.scene,
                    replay_delta,
                    |ops| {
                        self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                    },
                );
            }

            // Selected group border overlay must remain ordered before edges (ADR 0082).
            let group_corner = Px(10.0 / zoom);
            let selected_groups = snapshot.selected_groups.clone();
            let _ = self.graph.read_ref(cx.app, |g| {
                for group_id in selected_groups {
                    let Some(group) = g.groups.get(&group_id) else {
                        continue;
                    };
                    let rect0 = self.group_rect_with_preview(group_id, group.rect);
                    let rect = Rect::new(
                        Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                        Size::new(Px(rect0.size.width), Px(rect0.size.height)),
                    );
                    if render_cull_rect.is_some_and(|c| !rects_intersect(rect, c)) {
                        continue;
                    }
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect,
                        background: self.style.group_background,
                        border: Edges::all(Px(1.0 / zoom)),
                        border_color: self.style.node_border_selected,
                        corner_radii: Corners::all(group_corner),
                    });
                }
            });

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
                .or_else(|| {
                    (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0])
                })
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
                            let hint = self.edge_render_hint(g, edge_id).normalized();
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
                            b.add_u64(geom_key.graph_rev);
                            b.add_u32(geom_key.zoom_bits);
                            b.add_u32(geom_key.node_origin_x_bits);
                            b.add_u32(geom_key.node_origin_y_bits);
                            b.add_u64(geom_key.draw_order_hash);
                            b.add_u64(geom_key.presenter_rev);
                            b.add_u64(geom_key.edge_types_rev);
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

                            let mut state = self
                                .edges_build_states
                                .remove(&tile_key)
                                .unwrap_or_else(|| {
                                    let render_edges: RenderData = self.collect_render_data(
                                        &*cx.app,
                                        &snapshot,
                                        geom.clone(),
                                        index.clone(),
                                        Some(tile_cull_rect),
                                        zoom,
                                        None,
                                        false,
                                        false,
                                        true,
                                    );
                                    EdgesBuildState {
                                        ops: vec![
                                            SceneOp::PushClipRect { rect: tile_rect },
                                            SceneOp::PopClip,
                                        ],
                                        edges: render_edges.edges,
                                        next_edge: 0,
                                    }
                                });

                            let mut tmp = fret_core::Scene::default();
                            let custom_paths =
                                self.collect_custom_edge_paths(&*cx.app, &state.edges, zoom);
                            let mut next_edge = state.next_edge.min(state.edges.len());
                            let mut tile_skipped = false;
                            for edge in state.edges.iter().skip(next_edge) {
                                let width = self.style.wire_width * edge.hint.width_mul.max(0.0);
                                let (stop, _marker_skipped) =
                                    if let Some(custom) = custom_paths.get(&edge.id) {
                                        let fallback = Point::new(
                                            Px(edge.to.x.0 - edge.from.x.0),
                                            Px(edge.to.y.0 - edge.from.y.0),
                                        );
                                        let (t0, t1) = path_start_end_tangents(&custom.commands)
                                            .unwrap_or((fallback, fallback));
                                        self.push_edge_custom_wire_and_markers_budgeted(
                                            &mut tmp,
                                            cx.services,
                                            custom.cache_key,
                                            &custom.commands,
                                            t0,
                                            t1,
                                            zoom,
                                            cx.scale_factor,
                                            edge.from,
                                            edge.to,
                                            edge.color,
                                            width,
                                            edge.hint.start_marker.as_ref(),
                                            edge.hint.end_marker.as_ref(),
                                            &mut wire_budget,
                                            &mut marker_budget,
                                            true,
                                        )
                                    } else {
                                        self.push_edge_wire_and_markers_budgeted(
                                            &mut tmp,
                                            cx.services,
                                            zoom,
                                            cx.scale_factor,
                                            edge.hint.route,
                                            edge.from,
                                            edge.to,
                                            edge.color,
                                            width,
                                            edge.hint.start_marker.as_ref(),
                                            edge.hint.end_marker.as_ref(),
                                            &mut wire_budget,
                                            &mut marker_budget,
                                            true,
                                        )
                                    };
                                if stop {
                                    tile_skipped = true;
                                    break;
                                }
                                next_edge = next_edge.saturating_add(1);
                            }

                            state.next_edge = next_edge;
                            if tile_skipped || state.next_edge < state.edges.len() {
                                skipped = true;
                            }

                            if state.edges.is_empty() {
                                self.edges_scene_cache.store_ops(tile_key, Vec::new());
                                continue;
                            }

                            if tmp.ops_len() > 0 {
                                match state.ops.pop() {
                                    Some(SceneOp::PopClip) => {
                                        state.ops.extend_from_slice(tmp.ops());
                                        state.ops.push(SceneOp::PopClip);
                                    }
                                    Some(other) => {
                                        state.ops.push(other);
                                        state.ops.extend_from_slice(tmp.ops());
                                    }
                                    None => {
                                        state.ops.extend_from_slice(tmp.ops());
                                    }
                                }
                                if !matches!(state.ops.last(), Some(SceneOp::PopClip)) {
                                    state.ops.push(SceneOp::PopClip);
                                }
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

                        self.paint_edge_overlays_selected_hovered(cx, &snapshot, &geom, zoom);
                    } else {
                        self.edges_build_states.clear();
                        let render_edges: RenderData = self.collect_render_data(
                            &*cx.app,
                            &snapshot,
                            geom.clone(),
                            index.clone(),
                            Some(edges_rect),
                            zoom,
                            hovered_edge,
                            false,
                            false,
                            true,
                        );
                        self.paint_edges(
                            cx,
                            &snapshot,
                            &render_edges,
                            &geom,
                            zoom,
                            view_interacting,
                        );
                    }

                    // --- Edge labels (static, cached tiles) ---
                    self.edge_labels_tile_keys_scratch.clear();

                    let labels_base_key = {
                        let mut b =
                            TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.tile.v1");
                        b.add_u64(geom_key.graph_rev);
                        b.add_u32(geom_key.zoom_bits);
                        b.add_u32(geom_key.node_origin_x_bits);
                        b.add_u32(geom_key.node_origin_y_bits);
                        b.add_u64(geom_key.draw_order_hash);
                        b.add_u64(geom_key.presenter_rev);
                        b.add_u64(geom_key.edge_types_rev);
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
                                let render_edges: RenderData = self.collect_render_data(
                                    &*cx.app,
                                    &snapshot,
                                    geom.clone(),
                                    index.clone(),
                                    Some(tile_cull_rect),
                                    zoom,
                                    None,
                                    false,
                                    false,
                                    true,
                                );
                                EdgeLabelsBuildState {
                                    key: tile_key,
                                    ops: vec![
                                        SceneOp::PushClipRect { rect: tile_rect },
                                        SceneOp::PopClip,
                                    ],
                                    edges: render_edges.edges,
                                    next_edge: 0,
                                }
                            });

                        if state.edges.is_empty() {
                            self.edge_labels_scene_cache.store_ops(tile_key, Vec::new());
                            continue;
                        }

                        let mut tmp = fret_core::Scene::default();
                        let custom_paths =
                            self.collect_custom_edge_paths(&*cx.app, &state.edges, zoom);
                        let bezier_steps =
                            usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
                        let (next_edge, tile_skipped) = self.paint_edge_labels_static_budgeted(
                            &mut tmp,
                            cx.services,
                            cx.scale_factor,
                            &state.edges,
                            (!custom_paths.is_empty()).then_some(&custom_paths),
                            bezier_steps,
                            zoom,
                            state.next_edge,
                            &mut label_budget,
                        );
                        state.next_edge = next_edge;
                        if tile_skipped || state.next_edge < state.edges.len() {
                            skipped_labels = true;
                        }

                        if tmp.ops_len() > 0 {
                            match state.ops.pop() {
                                Some(SceneOp::PopClip) => {
                                    state.ops.extend_from_slice(tmp.ops());
                                    state.ops.push(SceneOp::PopClip);
                                }
                                Some(other) => {
                                    state.ops.push(other);
                                    state.ops.extend_from_slice(tmp.ops());
                                }
                                None => {
                                    state.ops.extend_from_slice(tmp.ops());
                                }
                            }
                            if !matches!(state.ops.last(), Some(SceneOp::PopClip)) {
                                state.ops.push(SceneOp::PopClip);
                            }
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
                        b.add_u64(geom_key.graph_rev);
                        b.add_u32(geom_key.zoom_bits);
                        b.add_u32(geom_key.node_origin_x_bits);
                        b.add_u32(geom_key.node_origin_y_bits);
                        b.add_u64(geom_key.draw_order_hash);
                        b.add_u64(geom_key.presenter_rev);
                        b.add_u64(geom_key.edge_types_rev);
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
                            &snapshot,
                            geom.clone(),
                            index.clone(),
                            render_cull_rect,
                            zoom,
                            hovered_edge,
                            false,
                            false,
                            true,
                        );
                        self.paint_edges(
                            cx,
                            &snapshot,
                            &render_edges,
                            &geom,
                            zoom,
                            view_interacting,
                        );
                    }

                    let labels_key = {
                        let mut b =
                            TileCacheKeyBuilder::new("fret-node.canvas.static_edge_labels.v1");
                        b.add_u64(geom_key.graph_rev);
                        b.add_u32(geom_key.zoom_bits);
                        b.add_u32(geom_key.node_origin_x_bits);
                        b.add_u32(geom_key.node_origin_y_bits);
                        b.add_u64(geom_key.draw_order_hash);
                        b.add_u64(geom_key.presenter_rev);
                        b.add_u64(geom_key.edge_types_rev);
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
                            let mut state = self
                                .edges_build_states
                                .remove(&edges_key)
                                .unwrap_or_else(|| {
                                    let render_edges: RenderData = self.collect_render_data(
                                        &*cx.app,
                                        &snapshot,
                                        geom.clone(),
                                        index.clone(),
                                        Some(edges_cache_rect),
                                        zoom,
                                        None,
                                        false,
                                        false,
                                        true,
                                    );
                                    EdgesBuildState {
                                        ops: vec![
                                            SceneOp::PushClipRect {
                                                rect: edges_cache_rect,
                                            },
                                            SceneOp::PopClip,
                                        ],
                                        edges: render_edges.edges,
                                        next_edge: 0,
                                    }
                                });

                            let wire_budget_limit =
                                Self::EDGE_WIRE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
                            let marker_budget_limit =
                                Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
                            let mut wire_budget = WorkBudget::new(wire_budget_limit);
                            let mut marker_budget = WorkBudget::new(marker_budget_limit);

                            let mut tmp = fret_core::Scene::default();
                            let custom_paths =
                                self.collect_custom_edge_paths(&*cx.app, &state.edges, zoom);
                            let mut next_edge = state.next_edge.min(state.edges.len());
                            let mut skipped = false;

                            for edge in state.edges.iter().skip(next_edge) {
                                let width = self.style.wire_width * edge.hint.width_mul.max(0.0);
                                let (stop, _marker_skipped) =
                                    if let Some(custom) = custom_paths.get(&edge.id) {
                                        let fallback = Point::new(
                                            Px(edge.to.x.0 - edge.from.x.0),
                                            Px(edge.to.y.0 - edge.from.y.0),
                                        );
                                        let (t0, t1) = path_start_end_tangents(&custom.commands)
                                            .unwrap_or((fallback, fallback));
                                        self.push_edge_custom_wire_and_markers_budgeted(
                                            &mut tmp,
                                            cx.services,
                                            custom.cache_key,
                                            &custom.commands,
                                            t0,
                                            t1,
                                            zoom,
                                            cx.scale_factor,
                                            edge.from,
                                            edge.to,
                                            edge.color,
                                            width,
                                            edge.hint.start_marker.as_ref(),
                                            edge.hint.end_marker.as_ref(),
                                            &mut wire_budget,
                                            &mut marker_budget,
                                            true,
                                        )
                                    } else {
                                        self.push_edge_wire_and_markers_budgeted(
                                            &mut tmp,
                                            cx.services,
                                            zoom,
                                            cx.scale_factor,
                                            edge.hint.route,
                                            edge.from,
                                            edge.to,
                                            edge.color,
                                            width,
                                            edge.hint.start_marker.as_ref(),
                                            edge.hint.end_marker.as_ref(),
                                            &mut wire_budget,
                                            &mut marker_budget,
                                            true,
                                        )
                                    };
                                if stop {
                                    skipped = true;
                                    break;
                                }
                                next_edge = next_edge.saturating_add(1);
                            }

                            state.next_edge = next_edge;
                            if skipped || state.next_edge < state.edges.len() {
                                cx.request_redraw();
                            }

                            if tmp.ops_len() > 0 {
                                match state.ops.pop() {
                                    Some(SceneOp::PopClip) => {
                                        state.ops.extend_from_slice(tmp.ops());
                                        state.ops.push(SceneOp::PopClip);
                                    }
                                    Some(other) => {
                                        state.ops.push(other);
                                        state.ops.extend_from_slice(tmp.ops());
                                    }
                                    None => {
                                        state.ops.extend_from_slice(tmp.ops());
                                    }
                                }
                                if !matches!(state.ops.last(), Some(SceneOp::PopClip)) {
                                    state.ops.push(SceneOp::PopClip);
                                }
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
                                let render_edges: RenderData = self.collect_render_data(
                                    &*cx.app,
                                    &snapshot,
                                    geom.clone(),
                                    index.clone(),
                                    Some(edges_cache_rect),
                                    zoom,
                                    None,
                                    false,
                                    false,
                                    true,
                                );
                                EdgeLabelsBuildState {
                                    key: labels_key,
                                    ops: vec![
                                        SceneOp::PushClipRect {
                                            rect: edges_cache_rect,
                                        },
                                        SceneOp::PopClip,
                                    ],
                                    edges: render_edges.edges,
                                    next_edge: 0,
                                }
                            });

                        let budget_limit =
                            Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
                        let mut budget = WorkBudget::new(budget_limit);

                        let mut tmp = fret_core::Scene::default();
                        let custom_paths =
                            self.collect_custom_edge_paths(&*cx.app, &state.edges, zoom);
                        let bezier_steps =
                            usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
                        let (next_edge, skipped) = self.paint_edge_labels_static_budgeted(
                            &mut tmp,
                            cx.services,
                            cx.scale_factor,
                            &state.edges,
                            (!custom_paths.is_empty()).then_some(&custom_paths),
                            bezier_steps,
                            zoom,
                            state.next_edge,
                            &mut budget,
                        );
                        state.next_edge = next_edge;

                        if skipped || state.next_edge < state.edges.len() {
                            cx.request_redraw();
                        }

                        if tmp.ops_len() > 0 {
                            match state.ops.pop() {
                                Some(SceneOp::PopClip) => {
                                    state.ops.extend_from_slice(tmp.ops());
                                    state.ops.push(SceneOp::PopClip);
                                }
                                Some(other) => {
                                    state.ops.push(other);
                                    state.ops.extend_from_slice(tmp.ops());
                                }
                                None => {
                                    state.ops.extend_from_slice(tmp.ops());
                                }
                            }
                            if !matches!(state.ops.last(), Some(SceneOp::PopClip)) {
                                state.ops.push(SceneOp::PopClip);
                            }
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
                        self.paint_edge_overlays_selected_hovered(cx, &snapshot, &geom, zoom);

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
                    &snapshot,
                    geom.clone(),
                    index.clone(),
                    render_cull_rect,
                    zoom,
                    hovered_edge,
                    false,
                    false,
                    true,
                );
                self.paint_edges(cx, &snapshot, &render_edges, &geom, zoom, view_interacting);
            }

            // --- Nodes (static, cached) ---
            let nodes_key = {
                let mut b = TileCacheKeyBuilder::new("fret-node.canvas.static_nodes.v1");
                b.add_u64(geom_key.graph_rev);
                b.add_u32(geom_key.zoom_bits);
                b.add_u32(geom_key.node_origin_x_bits);
                b.add_u32(geom_key.node_origin_y_bits);
                b.add_u64(geom_key.draw_order_hash);
                b.add_u64(geom_key.presenter_rev);
                b.add_u64(geom_key.edge_types_rev);
                b.add_u64(style_key);
                b.add_f32_bits(nodes_cache_tile_size_canvas);
                b.add_u32(cache_rect.origin.x.0.to_bits());
                b.add_u32(cache_rect.origin.y.0.to_bits());
                b.finish()
            };

            let nodes_hit =
                self.nodes_scene_cache
                    .try_replay_with(nodes_key, cx.scene, replay_delta, |ops| {
                        self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                    });
            if !nodes_hit {
                let render_nodes: RenderData = self.collect_render_data(
                    &*cx.app,
                    &snapshot,
                    geom.clone(),
                    index.clone(),
                    Some(cache_rect),
                    zoom,
                    None,
                    false,
                    true,
                    false,
                );

                let mut tmp = fret_core::Scene::default();
                tmp.push(SceneOp::PushClipRect { rect: cache_rect });
                self.paint_nodes_static(
                    &mut tmp,
                    cx.services,
                    cx.scale_factor,
                    &render_nodes,
                    zoom,
                );
                tmp.push(SceneOp::PopClip);
                self.nodes_scene_cache
                    .store_ops(nodes_key, tmp.ops().to_vec());
                let _ = self.nodes_scene_cache.try_replay_with(
                    nodes_key,
                    cx.scene,
                    replay_delta,
                    |ops| {
                        self.paint_cache.touch_text_blobs_in_scene_ops(ops);
                    },
                );
            }

            if snapshot.interaction.elevate_nodes_on_select {
                let render_selected = self.collect_selected_nodes_render_data(
                    &*cx.app,
                    &snapshot,
                    &geom,
                    render_cull_rect,
                    zoom,
                );
                if !render_selected.nodes.is_empty() {
                    self.paint_nodes_static(
                        cx.scene,
                        cx.services,
                        cx.scale_factor,
                        &render_selected,
                        zoom,
                    );
                }
            }

            // --- Nodes (dynamic overlays) ---
            self.paint_nodes_dynamic_from_geometry(cx, &snapshot, &geom, zoom);

            self.paint_edge_focus_anchors(
                cx,
                &snapshot,
                edge_anchor_target_id,
                edge_anchor_target,
                zoom,
            );
            self.paint_overlays(
                cx,
                &snapshot,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_w,
                viewport_h,
            );

            let prune = snapshot.interaction.paint_cache_prune;
            self.groups_scene_cache.prune(
                Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
                Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
            );
            self.nodes_scene_cache.prune(
                Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
                Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
            );
            self.edges_scene_cache.prune(
                Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
                Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
            );
            self.edge_labels_scene_cache.prune(
                Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
                Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
            );
            if prune.max_entries > 0 && prune.max_age_frames > 0 {
                self.paint_cache
                    .prune(cx.services, prune.max_age_frames, prune.max_entries);
                let tile_budget = (prune.max_entries / 10).clamp(64, 2048);
                self.grid_scene_cache
                    .prune(prune.max_age_frames, tile_budget);
            }

            cx.scene.push(SceneOp::PopClip);
            return;
        }

        // Fallback: immediate-mode paint (no static scene replay cache).
        let render: RenderData = self.collect_render_data(
            &*cx.app,
            &snapshot,
            geom.clone(),
            index.clone(),
            render_cull_rect,
            zoom,
            hovered_edge,
            true,
            true,
            true,
        );

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
            edge_anchor_target_id.and_then(|id| {
                render
                    .edges
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| (e.hint.route, e.from, e.to, e.color))
            });

        self.paint_groups_static(cx.scene, cx.services, cx.scale_factor, &render.groups, zoom);
        self.paint_groups_selected_overlay(cx.scene, &render.groups, zoom);
        self.paint_edges(cx, &snapshot, &render, &geom, zoom, view_interacting);
        self.paint_nodes_static(cx.scene, cx.services, cx.scale_factor, &render, zoom);
        if snapshot.interaction.elevate_nodes_on_select {
            let render_selected = self.collect_selected_nodes_render_data(
                &*cx.app,
                &snapshot,
                &geom,
                render_cull_rect,
                zoom,
            );
            if !render_selected.nodes.is_empty() {
                self.paint_nodes_static(
                    cx.scene,
                    cx.services,
                    cx.scale_factor,
                    &render_selected,
                    zoom,
                );
            }
        }
        self.paint_nodes_dynamic_from_geometry(cx, &snapshot, &geom, zoom);
        self.paint_edge_focus_anchors(
            cx,
            &snapshot,
            edge_anchor_target_id,
            edge_anchor_target,
            zoom,
        );
        self.paint_overlays(
            cx,
            &snapshot,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );

        let prune = snapshot.interaction.paint_cache_prune;
        self.groups_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.nodes_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.edges_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        self.edge_labels_scene_cache.prune(
            Self::STATIC_SCENE_TILE_CACHE_MAX_AGE_FRAMES,
            Self::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
        );
        if prune.max_entries > 0 && prune.max_age_frames > 0 {
            self.paint_cache
                .prune(cx.services, prune.max_age_frames, prune.max_entries);
            let tile_budget = (prune.max_entries / 10).clamp(64, 2048);
            self.grid_scene_cache
                .prune(prune.max_age_frames, tile_budget);
        }

        cx.scene.push(SceneOp::PopClip);
    }

    pub(super) fn paint_searcher<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        searcher: &SearcherState,
        zoom: f32,
    ) {
        let visible_rows = searcher_visible_rows(searcher);
        let rect = searcher_rect_at(&self.style, searcher.origin, visible_rows, zoom);
        let border_w = Px(1.0 / zoom);
        let radius = Px(self.style.context_menu_corner_radius / zoom);

        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(55),
            rect,
            background: self.style.context_menu_background,
            border: Edges::all(border_w),
            border_color: self.style.context_menu_border,
            corner_radii: Corners::all(radius),
        });

        let pad = self.style.context_menu_padding / zoom;
        let item_h = self.style.context_menu_item_height / zoom;
        let inner_x = rect.origin.x.0 + pad;
        let inner_y = rect.origin.y.0 + pad;
        let inner_w = (rect.size.width.0 - 2.0 * pad).max(0.0);

        let mut text_style = self.style.context_menu_text_style.clone();
        text_style.size = Px(text_style.size.0 / zoom);
        if let Some(lh) = text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let constraints = TextConstraints {
            max_width: Some(Px(inner_w)),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            scale_factor: effective_scale_factor(cx.scale_factor, zoom),
        };

        let query_rect = Rect::new(
            Point::new(Px(inner_x), Px(inner_y)),
            Size::new(Px(inner_w), Px(item_h)),
        );
        cx.scene.push(SceneOp::Quad {
            order: DrawOrder(56),
            rect: query_rect,
            background: self.style.context_menu_hover_background,
            border: Edges::all(Px(0.0)),
            border_color: Color::TRANSPARENT,
            corner_radii: Corners::all(Px(4.0 / zoom)),
        });

        let query_text = if searcher.query.is_empty() {
            Arc::<str>::from("Search...")
        } else {
            Arc::<str>::from(format!("Search: {}", searcher.query))
        };
        let (blob, metrics) =
            self.paint_cache
                .text_blob(cx.services, query_text, &text_style, constraints);
        let text_x = query_rect.origin.x;
        let text_y = Px(query_rect.origin.y.0
            + (query_rect.size.height.0 - metrics.size.height.0) * 0.5
            + metrics.baseline.0);
        let query_color = if searcher.query.is_empty() {
            self.style.context_menu_text_disabled
        } else {
            self.style.context_menu_text
        };
        cx.scene.push(SceneOp::Text {
            order: DrawOrder(57),
            origin: Point::new(text_x, text_y),
            text: blob,
            color: query_color,
        });

        let list_y0 = inner_y + item_h + pad;
        let start = searcher.scroll.min(searcher.rows.len());
        let end = (start + visible_rows).min(searcher.rows.len());
        for (slot, row_ix) in (start..end).enumerate() {
            let row = &searcher.rows[row_ix];
            let item_rect = Rect::new(
                Point::new(Px(inner_x), Px(list_y0 + slot as f32 * item_h)),
                Size::new(Px(inner_w), Px(item_h)),
            );

            let is_active = searcher.active_row == row_ix;
            let is_hovered = searcher.hovered_row == Some(row_ix);
            if (is_hovered || is_active) && Self::searcher_is_selectable_row(row) {
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(56),
                    rect: item_rect,
                    background: self.style.context_menu_hover_background,
                    border: Edges::all(Px(0.0)),
                    border_color: Color::TRANSPARENT,
                    corner_radii: Corners::all(Px(4.0 / zoom)),
                });
            }

            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                row.label.clone(),
                &text_style,
                constraints,
            );

            let text_x = item_rect.origin.x;
            let text_y = Px(item_rect.origin.y.0
                + (item_rect.size.height.0 - metrics.size.height.0) * 0.5
                + metrics.baseline.0);
            let color = if row.enabled {
                self.style.context_menu_text
            } else {
                self.style.context_menu_text_disabled
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(57),
                origin: Point::new(text_x, text_y),
                text: blob,
                color,
            });
        }
    }
}
