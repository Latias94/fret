use super::super::*;

#[path = "cached_edges/mod.rs"]
mod cached_edges;
#[path = "cached_groups.rs"]
mod cached_groups;
#[path = "cached_nodes.rs"]
mod cached_nodes;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    const STATIC_NODES_TILE_MUL: f32 = 2.0;
    const STATIC_SCENE_TILE_SIZE_SCREEN_PX_MIN: u32 = 1024;
    const STATIC_EDGES_TILE_SIZE_SCREEN_PX: u32 = 2048;

    fn next_power_of_two_at_least(min: u32, value: f32) -> u32 {
        let target = value.ceil().max(1.0) as u32;
        let pow2 = target.checked_next_power_of_two().unwrap_or(0x8000_0000);
        pow2.max(min)
    }

    pub(in super::super) fn paint_root<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
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

        let viewport = Self::viewport_from_pan_zoom(cx.bounds, pan, zoom);
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
        self.publish_derived_outputs(&*cx.app, &snapshot, cx.bounds, &geom);

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

        let style_key = self.static_scene_style_key(cx.scale_factor);

        let geom_key = self
            .geometry
            .geom_key
            .unwrap_or_else(|| self.geometry_key(&*cx.app, &snapshot));
        let base_key = geom_key.base;

        if let Some(cache_rect) = nodes_cache_rect {
            self.paint_root_groups_cached_path(
                cx,
                &snapshot,
                &geom,
                &index,
                cache_rect,
                render_cull_rect,
                zoom,
                base_key,
                style_key,
                nodes_cache_tile_size_canvas,
            );

            // --- Edges (static + overlays) ---
            let (edge_anchor_target_id, edge_anchor_target) = self.paint_root_edges_cached_path(
                cx,
                &snapshot,
                &geom,
                &index,
                hovered_edge,
                cache_rect,
                edges_cache_rect,
                render_cull_rect,
                viewport_rect,
                viewport_w,
                viewport_h,
                zoom,
                view_interacting,
                base_key,
                style_key,
                edges_cache_tile_size_canvas,
            );

            self.paint_root_nodes_cached_path(
                cx,
                &snapshot,
                &geom,
                &index,
                cache_rect,
                render_cull_rect,
                zoom,
                base_key,
                style_key,
                nodes_cache_tile_size_canvas,
            );

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

            self.prune_paint_caches(cx.services, &snapshot);

            cx.scene.push(SceneOp::PopClip);
            return;
        }

        self.paint_root_immediate_path(
            cx,
            &snapshot,
            geom,
            index,
            hovered_edge,
            render_cull_rect,
            view_interacting,
            zoom,
            viewport_origin_x,
            viewport_origin_y,
            viewport_w,
            viewport_h,
        );
    }
}
