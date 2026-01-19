use super::paint_render_data::RenderData;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        zoom: f32,
        view_interacting: bool,
    ) {
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

        let edge_insert_marker_request = self
            .interaction
            .edge_insert_drag
            .as_ref()
            .map(|d| (d.edge, d.pos));
        let insert_node_drag_preview = self.interaction.insert_node_drag_preview.as_ref();

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

        for edge in &render.edges {
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
            let hovered_port = self.interaction.hover_port;
            let hovered_port_valid = self.interaction.hover_port_valid;
            let hovered_port_convertible = self.interaction.hover_port_convertible;
            let focused_port = self.interaction.focused_port;
            let focused_port_valid = self.interaction.focused_port_valid;
            let focused_port_convertible = self.interaction.focused_port_convertible;

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
    }
}
