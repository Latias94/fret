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
        let hovered_port = self.interaction.hover_port;
        let hovered_port_valid = self.interaction.hover_port_valid;
        let hovered_port_convertible = self.interaction.hover_port_convertible;
        let focused_port = self.interaction.focused_port;
        let focused_port_valid = self.interaction.focused_port_valid;
        let focused_port_convertible = self.interaction.focused_port_convertible;
        let wire_drag = self.interaction.wire_drag.clone();
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.clone();
        let edge_insert_marker_request = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|d| (d.edge, d.pos));
        let marked_ports: HashSet<PortId> = match wire_drag.as_ref().map(|w| &w.kind) {
            Some(WireDragKind::New { bundle, .. }) if bundle.len() > 1 => {
                bundle.iter().copied().collect()
            }
            Some(WireDragKind::ReconnectMany { edges }) if edges.len() > 1 => edges
                .iter()
                .map(|(_edge, _endpoint, fixed)| *fixed)
                .collect(),
            _ => HashSet::new(),
        };

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

        // Groups render under edges and nodes (container frames).
        if !render.groups.is_empty() {
            let mut group_text_style = self.style.context_menu_text_style.clone();
            group_text_style.size = Px(group_text_style.size.0 / zoom);
            if let Some(lh) = group_text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }

            let group_pad = 10.0 / zoom;
            let group_corner = Px(10.0 / zoom);
            for (rect, title, selected) in &render.groups {
                let border_color = if *selected {
                    self.style.node_border_selected
                } else {
                    self.style.group_border
                };
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(1),
                    rect: *rect,
                    background: self.style.group_background,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(group_corner),
                });

                if !title.is_empty() {
                    let max_w = (rect.size.width.0 - 2.0 * group_pad).max(0.0);
                    let constraints = TextConstraints {
                        max_width: Some(Px(max_w)),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        scale_factor: effective_scale_factor(cx.scale_factor, zoom),
                    };
                    let (blob, metrics) = self.paint_cache.text_blob(
                        cx.services,
                        title.clone(),
                        &group_text_style,
                        constraints,
                    );

                    let text_x = Px(rect.origin.x.0 + group_pad);
                    let text_y = Px(rect.origin.y.0 + group_pad + metrics.baseline.0);
                    cx.scene.push(SceneOp::Text {
                        order: DrawOrder(1),
                        origin: Point::new(text_x, text_y),
                        text: blob,
                        color: self.style.context_menu_text,
                    });
                }
            }
        }

        #[derive(Debug, Clone)]
        struct EdgePaint {
            from: Point,
            to: Point,
            color: Color,
            width: f32,
            route: EdgeRouteKind,
            start_marker: Option<crate::ui::presenter::EdgeMarker>,
            end_marker: Option<crate::ui::presenter::EdgeMarker>,
        }

        let mut edges_normal: Vec<EdgePaint> = Vec::new();
        let mut edges_selected: Vec<EdgePaint> = Vec::new();
        let mut edges_hovered: Vec<EdgePaint> = Vec::new();
        let mut edge_labels: Vec<(Point, Point, EdgeRouteKind, Arc<str>, bool, bool)> = Vec::new();

        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let edge_insert_marker: Option<(Point, Color)> =
            edge_insert_marker_request.and_then(|(edge_id, pos)| {
                render.edges.iter().find(|e| e.id == edge_id).map(|e| {
                    (
                        closest_point_on_edge_route(
                            e.hint.route,
                            e.from,
                            e.to,
                            zoom,
                            bezier_steps,
                            pos,
                        ),
                        e.color,
                    )
                })
            });

        let insert_node_drag_marker: Option<(Point, Color)> =
            insert_node_drag_preview.as_ref().map(|p| {
                if let Some(edge_id) = p.edge
                    && let Some(edge) = render.edges.iter().find(|e| e.id == edge_id)
                {
                    (
                        closest_point_on_edge_route(
                            edge.hint.route,
                            edge.from,
                            edge.to,
                            zoom,
                            bezier_steps,
                            p.pos,
                        ),
                        edge.color,
                    )
                } else {
                    (p.pos, self.style.wire_color_preview)
                }
            });

        for edge in render.edges {
            let mut width = self.style.wire_width * edge.hint.width_mul.max(0.0);
            if edge.selected {
                width *= self.style.wire_width_selected_mul;
            }
            if edge.hovered {
                width *= self.style.wire_width_hover_mul;
            }

            let route = edge.hint.route;
            if let Some(label) = edge.hint.label.as_ref().filter(|s| !s.is_empty()) {
                edge_labels.push((
                    edge.from,
                    edge.to,
                    route,
                    label.clone(),
                    edge.selected,
                    edge.hovered,
                ));
            }

            let paint = EdgePaint {
                from: edge.from,
                to: edge.to,
                color: edge.color,
                width,
                route,
                start_marker: edge.hint.start_marker.clone(),
                end_marker: edge.hint.end_marker.clone(),
            };

            if edge.hovered {
                edges_hovered.push(paint);
            } else if edge.selected {
                edges_selected.push(paint);
            } else {
                edges_normal.push(paint);
            }
        }

        let marker_budget_limit = Self::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut marker_budget = WorkBudget::new(marker_budget_limit);
        let mut marker_budget_skipped: u32 = 0;

        for edge in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            let path = self.paint_cache.wire_path(
                cx.services,
                edge.route,
                edge.from,
                edge.to,
                zoom,
                cx.scale_factor,
                edge.width,
            );

            if let Some(path) = path {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color: edge.color,
                });
            }

            if let Some(marker) = edge.end_marker.as_ref() {
                let (path, skipped_by_budget) = self.paint_cache.edge_end_marker_path_budgeted(
                    cx.services,
                    edge.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                    &mut marker_budget,
                );
                if skipped_by_budget {
                    marker_budget_skipped = marker_budget_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color: edge.color,
                    });
                }
            }

            if let Some(marker) = edge.start_marker.as_ref() {
                let (path, skipped_by_budget) = self.paint_cache.edge_start_marker_path_budgeted(
                    cx.services,
                    edge.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                    &mut marker_budget,
                );
                if skipped_by_budget {
                    marker_budget_skipped = marker_budget_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color: edge.color,
                    });
                }
            }
        }

        if marker_budget_skipped > 0 {
            cx.request_redraw();
        }

        let prune = snapshot.interaction.paint_cache_prune;
        if prune.max_entries > 0 && prune.max_age_frames > 0 {
            self.paint_cache
                .prune(cx.services, prune.max_age_frames, prune.max_entries);
            let tile_budget = (prune.max_entries / 10).clamp(64, 2048);
            self.grid_scene_cache
                .prune(prune.max_age_frames, tile_budget);
        }

        let mut draw_drop_marker = |pos: Point, color: Color| {
            let z = zoom.max(1.0e-6);
            let r = 7.0 / z;
            let border_w = 2.0 / z;
            let rect = Rect::new(
                Point::new(Px(pos.x.0 - r), Px(pos.y.0 - r)),
                Size::new(Px(2.0 * r), Px(2.0 * r)),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: Color::TRANSPARENT,
                border: Edges::all(Px(border_w)),
                border_color: color,
                corner_radii: Corners::all(Px(r)),
            });

            let arm = 10.0 / z;
            let thick = (2.0 / z).max(0.5 / z);
            let h_rect = Rect::new(
                Point::new(Px(pos.x.0 - arm * 0.5), Px(pos.y.0 - thick * 0.5)),
                Size::new(Px(arm), Px(thick)),
            );
            let v_rect = Rect::new(
                Point::new(Px(pos.x.0 - thick * 0.5), Px(pos.y.0 - arm * 0.5)),
                Size::new(Px(thick), Px(arm)),
            );
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect: h_rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect: v_rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(Px(0.0)),
            });
        };

        if let Some((pos, color)) = edge_insert_marker {
            draw_drop_marker(pos, color);
        }
        if let Some((pos, color)) = insert_node_drag_marker {
            draw_drop_marker(pos, color);
        }

        if !edge_labels.is_empty() {
            let pad_screen = 6.0;
            let corner_screen = 8.0;
            let offset_screen = 10.0;

            let mut edge_text_style = self.style.context_menu_text_style.clone();
            edge_text_style.size = Px(edge_text_style.size.0 / zoom);
            if let Some(lh) = edge_text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }

            let label_budget_limit =
                Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
            let mut label_budget = WorkBudget::new(label_budget_limit);
            let mut label_budget_skipped: u32 = 0;
            for (from, to, route, label, _selected, _hovered) in edge_labels {
                let (pos, normal) = match route {
                    EdgeRouteKind::Bezier => {
                        let (c1, c2) = wire_ctrl_points(from, to, zoom);
                        let p = cubic_bezier(from, c1, c2, to, 0.5);
                        let d = cubic_bezier_derivative(from, c1, c2, to, 0.5);
                        (p, normal_from_tangent(d))
                    }
                    EdgeRouteKind::Straight => {
                        let p = Point::new(
                            Px(0.5 * (from.x.0 + to.x.0)),
                            Px(0.5 * (from.y.0 + to.y.0)),
                        );
                        let d = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
                        (p, normal_from_tangent(d))
                    }
                    EdgeRouteKind::Step => {
                        let mx = 0.5 * (from.x.0 + to.x.0);
                        let p = Point::new(Px(mx), Px(0.5 * (from.y.0 + to.y.0)));
                        (p, Point::new(Px(0.0), Px(-1.0)))
                    }
                };

                let z = zoom.max(1.0e-6);
                let off = offset_screen / z;
                let anchor = Point::new(
                    Px(pos.x.0 + normal.x.0 * off),
                    Px(pos.y.0 + normal.y.0 * off),
                );

                let max_w = 220.0 / z;
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    scale_factor: cx.scale_factor * zoom,
                };

                let (prepared, skipped_by_budget) = self.paint_cache.text_blob_budgeted(
                    cx.services,
                    label.clone(),
                    &edge_text_style,
                    constraints,
                    &mut label_budget,
                );
                if skipped_by_budget {
                    label_budget_skipped = label_budget_skipped.saturating_add(1);
                    cx.request_redraw();
                    break;
                }
                let Some((blob, metrics)) = prepared else {
                    continue;
                };

                let pad = pad_screen / z;
                let w = metrics.size.width.0.max(0.0);
                let h = metrics.size.height.0.max(0.0);
                let rect = Rect::new(
                    Point::new(
                        Px(anchor.x.0 - 0.5 * w - pad),
                        Px(anchor.y.0 - 0.5 * h - pad),
                    ),
                    Size::new(Px(w + 2.0 * pad), Px(h + 2.0 * pad)),
                );

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(2),
                    rect,
                    background: self.style.context_menu_background,
                    border: Edges::all(Px(1.0 / z)),
                    border_color: self.style.context_menu_border,
                    corner_radii: Corners::all(Px(corner_screen / z)),
                });

                let text_x = Px(rect.origin.x.0 + pad);
                let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(2),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }

            if let Some(window) = cx.window {
                let frame_id = cx.app.frame_id().0;
                let key = CanvasCacheKey {
                    window: window.data().as_ffi(),
                    node: cx.node.data().as_ffi(),
                    name: "fret-node.canvas.edge_labels_budget",
                };
                cx.app
                    .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                        registry.record_work_budget(
                            key,
                            frame_id,
                            label_budget.used().saturating_add(label_budget_skipped),
                            label_budget_limit,
                            label_budget.used(),
                            label_budget_skipped,
                        );
                    });
            }
        }
        if let Some(window) = cx.window {
            let frame_id = cx.app.frame_id().0;
            let key = CanvasCacheKey {
                window: window.data().as_ffi(),
                node: cx.node.data().as_ffi(),
                name: "fret-node.canvas.edge_markers_budget",
            };
            cx.app
                .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                    registry.record_work_budget(
                        key,
                        frame_id,
                        marker_budget.used().saturating_add(marker_budget_skipped),
                        marker_budget_limit,
                        marker_budget.used(),
                        marker_budget_skipped,
                    );
                });
        }

        if let Some(w) = &self.interaction.wire_drag {
            let focused_target =
                focused_port.filter(|_| focused_port_valid || focused_port_convertible);
            let to = hovered_port
                .filter(|_| hovered_port_valid || hovered_port_convertible)
                .or(focused_target)
                .and_then(|port| render.port_centers.get(&port).copied())
                .unwrap_or(w.pos);
            let color =
                if hovered_port.is_some() && !hovered_port_valid && !hovered_port_convertible {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 0.95,
                    }
                } else if hovered_port.is_some() && hovered_port_convertible && !hovered_port_valid
                {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 0.95,
                    }
                } else if focused_port.is_some()
                    && !focused_port_valid
                    && !focused_port_convertible
                    && hovered_port.is_none()
                {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 0.95,
                    }
                } else if focused_port.is_some()
                    && focused_port_convertible
                    && !focused_port_valid
                    && hovered_port.is_none()
                {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 0.95,
                    }
                } else {
                    self.style.wire_color_preview
                };

            let mut draw_preview = |from: Point| {
                if let Some(path) = self.paint_cache.wire_path(
                    cx.services,
                    EdgeRouteKind::Bezier,
                    from,
                    to,
                    zoom,
                    cx.scale_factor,
                    self.style.wire_width,
                ) {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            };

            match &w.kind {
                WireDragKind::New { from, bundle } => {
                    let ports = if bundle.is_empty() {
                        std::slice::from_ref(from)
                    } else {
                        bundle.as_slice()
                    };
                    for port in ports {
                        if let Some(from) = render.port_centers.get(port).copied() {
                            draw_preview(from);
                        }
                    }
                }
                WireDragKind::Reconnect { fixed, .. } => {
                    if let Some(from) = render.port_centers.get(fixed).copied() {
                        draw_preview(from);
                    }
                }
                WireDragKind::ReconnectMany { edges } => {
                    for (_edge, _endpoint, fixed) in edges {
                        if let Some(from) = render.port_centers.get(fixed).copied() {
                            draw_preview(from);
                        }
                    }
                }
            }
        }

        let mut node_text_style = self.style.context_menu_text_style.clone();
        node_text_style.size = Px(node_text_style.size.0 / zoom);
        if let Some(lh) = node_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let corner = Px(8.0 / zoom);
        let title_pad = self.style.node_padding / zoom;
        let title_h = self.style.node_header_height / zoom;

        if let Some(preview) = insert_node_drag_preview.as_ref() {
            let z = zoom.max(1.0e-6);
            let w = self.style.node_width / z;
            let h = (self.style.node_header_height + 2.0 * self.style.pin_row_height) / z;
            let rect = Rect::new(
                Point::new(Px(preview.pos.x.0 - 0.5 * w), Px(preview.pos.y.0 - 0.5 * h)),
                Size::new(Px(w), Px(h)),
            );

            let mut bg = self.style.node_background;
            bg.a *= 0.55;
            let mut border_color = self.style.node_border_selected;
            border_color.a *= 0.85;
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: bg,
                border: Edges::all(Px(1.0 / z)),
                border_color,
                corner_radii: Corners::all(corner),
            });

            if !preview.label.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: cx.scale_factor * zoom,
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    preview.label.clone(),
                    &node_text_style,
                    constraints,
                );
                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        for (node, rect, is_selected, title, body, pin_rows, resize_handles) in &render.nodes {
            let rect = *rect;
            let border_color = if *is_selected {
                self.style.node_border_selected
            } else {
                self.style.node_border
            };
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(3),
                rect,
                background: self.style.node_background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color,
                corner_radii: Corners::all(corner),
            });

            let show_resize_handle = *is_selected
                && (self
                    .interaction
                    .node_resize
                    .as_ref()
                    .is_some_and(|r| r.node == *node)
                    || self
                        .interaction
                        .last_pos
                        .is_some_and(|p| Self::rect_contains(rect, p)));
            if show_resize_handle {
                for handle in NodeResizeHandle::ALL {
                    if !resize_handles.contains(handle) {
                        continue;
                    }
                    let rect = self.node_resize_handle_rect(rect, handle, zoom);
                    cx.scene.push(SceneOp::Quad {
                        order: DrawOrder(5),
                        rect,
                        background: self.style.resize_handle_background,
                        border: Edges::all(Px(1.0 / zoom)),
                        border_color: self.style.resize_handle_border,
                        corner_radii: Corners::all(Px(2.0 / zoom)),
                    });
                }
            }

            if !title.is_empty() {
                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: effective_scale_factor(cx.scale_factor, zoom),
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    title.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = rect.origin.y.0 + (title_h - metrics.size.height.0) * 0.5;
                let text_y = Px(inner_y + metrics.baseline.0);
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, text_y),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }

            if let Some(body) = body
                && !body.is_empty()
            {
                let pin_rows = (*pin_rows).max(0) as f32;
                let body_top = rect.origin.y.0
                    + (self.style.node_header_height
                        + self.style.node_padding
                        + pin_rows * self.style.pin_row_height
                        + self.style.node_padding)
                        / zoom;

                let max_w = (rect.size.width.0 - 2.0 * title_pad).max(0.0);
                let constraints = TextConstraints {
                    max_width: Some(Px(max_w)),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    scale_factor: effective_scale_factor(cx.scale_factor, zoom),
                };
                let (blob, metrics) = self.paint_cache.text_blob(
                    cx.services,
                    body.clone(),
                    &node_text_style,
                    constraints,
                );

                let text_x = Px(rect.origin.x.0 + title_pad);
                let inner_y = body_top + metrics.baseline.0;
                cx.scene.push(SceneOp::Text {
                    order: DrawOrder(4),
                    origin: Point::new(text_x, Px(inner_y)),
                    text: blob,
                    color: self.style.context_menu_text,
                });
            }
        }

        let pin_r = self.style.pin_radius / zoom;
        let pin_gap = 8.0 / zoom;

        for (port_id, info) in &render.port_labels {
            let Some(center) = render.port_centers.get(port_id).copied() else {
                continue;
            };
            let port_constraints = TextConstraints {
                max_width: Some(info.max_width),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: effective_scale_factor(cx.scale_factor, zoom),
            };
            let (blob, metrics) = self.paint_cache.text_blob(
                cx.services,
                info.label.clone(),
                &node_text_style,
                port_constraints,
            );

            let y = Px(center.y.0 - 0.5 * metrics.size.height.0 + metrics.baseline.0);
            let x = match info.dir {
                PortDirection::In => Px(center.x.0 + pin_r + pin_gap),
                PortDirection::Out => Px(center.x.0 - pin_r - pin_gap - metrics.size.width.0),
            };

            cx.scene.push(SceneOp::Text {
                order: DrawOrder(4),
                origin: Point::new(x, y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        for (port_id, rect, color) in render.pins {
            if marked_ports.contains(&port_id) {
                let pad = 5.0 / zoom;
                let marker_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * marker_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: marker_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(1.0 / zoom)),
                    border_color: Color {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        a: 0.55,
                    },
                    corner_radii: Corners::all(r),
                });
            }

            if hovered_port == Some(port_id) {
                let border_color = if hovered_port_valid {
                    color
                } else if hovered_port_convertible {
                    Color {
                        r: 0.95,
                        g: 0.75,
                        b: 0.20,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.90,
                        g: 0.35,
                        b: 0.35,
                        a: 1.0,
                    }
                };
                let pad = 2.0 / zoom;
                let hover_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * hover_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: hover_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(2.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(r),
                });
            }

            if hovered_port != Some(port_id) && focused_port == Some(port_id) {
                let border_color = if self.interaction.wire_drag.is_some() {
                    if focused_port_valid {
                        color
                    } else if focused_port_convertible {
                        Color {
                            r: 0.95,
                            g: 0.75,
                            b: 0.20,
                            a: 1.0,
                        }
                    } else {
                        Color {
                            r: 0.90,
                            g: 0.35,
                            b: 0.35,
                            a: 1.0,
                        }
                    }
                } else {
                    self.style.node_border_selected
                };

                let pad = 2.0 / zoom;
                let hover_rect = Rect::new(
                    Point::new(Px(rect.origin.x.0 - pad), Px(rect.origin.y.0 - pad)),
                    Size::new(
                        Px(rect.size.width.0 + 2.0 * pad),
                        Px(rect.size.height.0 + 2.0 * pad),
                    ),
                );
                let r = Px(0.5 * hover_rect.size.width.0);
                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(4),
                    rect: hover_rect,
                    background: Color::TRANSPARENT,
                    border: Edges::all(Px(2.0 / zoom)),
                    border_color,
                    corner_radii: Corners::all(r),
                });
            }

            let r = Px(0.5 * rect.size.width.0);
            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(4),
                rect,
                background: color,
                border: Edges::all(Px(0.0)),
                border_color: Color::TRANSPARENT,
                corner_radii: Corners::all(r),
            });
        }

        if let Some((route, from, to, color)) = edge_anchor_target {
            let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
            let target_edge_id = edge_anchor_target_id;
            let (allow_from, allow_to) = target_edge_id
                .and_then(|edge_id| {
                    self.graph
                        .read_ref(cx.app, |g| {
                            let edge = g.edges.get(&edge_id)?;
                            Some(Self::edge_reconnectable_flags(edge, &snapshot.interaction))
                        })
                        .ok()
                        .flatten()
                })
                .unwrap_or((false, false));

            let z = zoom.max(1.0e-6);
            let border_base = Px(Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN / z);
            let anchor_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.95,
            };
            let fill_color = Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 0.15,
            };

            for (endpoint, center) in [(EdgeEndpoint::From, a0), (EdgeEndpoint::To, a1)] {
                if (endpoint == EdgeEndpoint::From && !allow_from)
                    || (endpoint == EdgeEndpoint::To && !allow_to)
                {
                    continue;
                }
                let rect = Self::edge_focus_anchor_rect(center, zoom);
                let r = Px(0.5 * rect.size.width.0);
                let hovered = self
                    .interaction
                    .hover_edge_anchor
                    .is_some_and(|(edge, ep)| Some(edge) == target_edge_id && ep == endpoint);
                let active = self
                    .interaction
                    .wire_drag
                    .as_ref()
                    .is_some_and(|w| match &w.kind {
                        WireDragKind::Reconnect {
                            edge, endpoint: ep, ..
                        } => Some(*edge) == target_edge_id && *ep == endpoint,
                        _ => false,
                    });

                let border = if active {
                    Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 1.0) / z)
                } else if hovered {
                    Px((Self::EDGE_FOCUS_ANCHOR_BORDER_SCREEN + 0.5) / z)
                } else {
                    border_base
                };

                let background = if active {
                    Color {
                        a: (fill_color.a + 0.20).min(1.0),
                        ..fill_color
                    }
                } else if hovered {
                    Color {
                        a: (fill_color.a + 0.10).min(1.0),
                        ..fill_color
                    }
                } else {
                    fill_color
                };

                cx.scene.push(SceneOp::Quad {
                    order: DrawOrder(6),
                    rect,
                    background,
                    border: Edges::all(border),
                    border_color: anchor_color,
                    corner_radii: Corners::all(r),
                });
            }
        }

        if self.close_command.is_some() {
            let rect = Self::close_button_rect(snapshot.pan, zoom);
            let hovered = self
                .interaction
                .last_pos
                .is_some_and(|p| Self::rect_contains(rect, p));

            let background = if hovered {
                self.style.context_menu_hover_background
            } else {
                self.style.context_menu_background
            };

            cx.scene.push(SceneOp::Quad {
                order: DrawOrder(60),
                rect,
                background,
                border: Edges::all(Px(1.0 / zoom)),
                border_color: self.style.context_menu_border,
                corner_radii: Corners::all(Px(6.0 / zoom)),
            });

            let mut text_style = self.style.context_menu_text_style.clone();
            text_style.size = Px(text_style.size.0 / zoom);
            if let Some(lh) = text_style.line_height.as_mut() {
                lh.0 /= zoom;
            }
            let pad = 10.0 / zoom;
            let constraints = TextConstraints {
                max_width: Some(Px((rect.size.width.0 - 2.0 * pad).max(0.0))),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                scale_factor: effective_scale_factor(cx.scale_factor, zoom),
            };
            let (blob, metrics) =
                self.paint_cache
                    .text_blob(cx.services, "Close", &text_style, constraints);

            let text_x = Px(rect.origin.x.0 + pad);
            let inner_y = rect.origin.y.0 + (rect.size.height.0 - metrics.size.height.0) * 0.5;
            let text_y = Px(inner_y + metrics.baseline.0);
            cx.scene.push(SceneOp::Text {
                order: DrawOrder(61),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        if let Some(wire_drag) = wire_drag {
            self.paint_wire_drag_hint(cx, &snapshot, &wire_drag, zoom);
        }

        if let Some(marquee) = self.interaction.marquee.clone() {
            self.paint_marquee(cx, &marquee, zoom);
        }

        if let Some(guides) = self.interaction.snap_guides {
            self.paint_snap_guides(
                cx,
                &guides,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_w,
                viewport_h,
            );
        }

        if let Some(searcher) = self.interaction.searcher.clone() {
            self.paint_searcher(cx, &searcher, zoom);
        }

        if let Some(menu) = self.interaction.context_menu.clone() {
            self.paint_context_menu(cx, &menu, zoom);
        }

        if let Some(toast) = self.interaction.toast.clone() {
            self.paint_toast(
                cx,
                &toast,
                zoom,
                viewport_origin_x,
                viewport_origin_y,
                viewport_h,
            );
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
