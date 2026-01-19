use super::paint_render_data::RenderData;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_root<H: UiHost>(&mut self, cx: &mut PaintCx<'_, H>) {
        cx.observe_model(&self.graph, Invalidation::Paint);
        cx.observe_model(&self.view_state, Invalidation::Paint);
        let snapshot = self.sync_view_state(cx.app);

        let view_interacting = self.view_interacting();

        self.paint_cache.begin_frame();
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
        for id in self.text_blobs.drain(..) {
            cx.services.text().release(id);
        }

        let zoom = snapshot.zoom;
        let pan = snapshot.pan;

        let viewport_w = cx.bounds.size.width.0 / zoom;
        let viewport_h = cx.bounds.size.height.0 / zoom;
        let viewport_origin_x = -pan.x;
        let viewport_origin_y = -pan.y;
        let viewport_rect = Rect::new(
            Point::new(Px(viewport_origin_x), Px(viewport_origin_y)),
            Size::new(Px(viewport_w), Px(viewport_h)),
        );
        let render_cull_rect = {
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
        let render: RenderData = self.collect_render_data(
            &*cx.app,
            &snapshot,
            geom.clone(),
            index.clone(),
            render_cull_rect,
            zoom,
            hovered_edge,
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

        self.paint_groups(cx, &render.groups, zoom);
        self.paint_edges(cx, &snapshot, &render, zoom, view_interacting);
        self.paint_nodes(cx, &render, zoom);
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
