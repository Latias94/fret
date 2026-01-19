use super::paint_render_data::{EdgeRender, RenderData};
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn push_edge_wire_and_markers_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        zoom: f32,
        scale_factor: f32,
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        color: Color,
        width: f32,
        start_marker: Option<&crate::ui::presenter::EdgeMarker>,
        end_marker: Option<&crate::ui::presenter::EdgeMarker>,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
        stop_on_marker_skip: bool,
    ) -> (bool, u32) {
        if !wire_budget.try_consume(1) {
            return (true, 0);
        }

        let mut marker_skipped: u32 = 0;

        let end_path = if stop_on_marker_skip {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_end_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        let start_path = if stop_on_marker_skip {
            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_start_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    return (true, 1);
                }
                path
            } else {
                None
            }
        } else {
            None
        };

        if let Some(path) =
            self.paint_cache
                .wire_path(services, route, from, to, zoom, scale_factor, width)
        {
            scene.push(SceneOp::Path {
                order: DrawOrder(2),
                origin: Point::new(Px(0.0), Px(0.0)),
                path,
                color,
            });
        }

        if stop_on_marker_skip {
            if let Some(path) = end_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
            if let Some(path) = start_path {
                scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color,
                });
            }
        } else {
            if let Some(marker) = end_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_end_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }

            if let Some(marker) = start_marker {
                let (path, skipped_by_budget) = self.paint_cache.edge_start_marker_path_budgeted(
                    services,
                    route,
                    from,
                    to,
                    zoom,
                    scale_factor,
                    marker,
                    self.style.pin_radius,
                    marker_budget,
                );
                if skipped_by_budget {
                    marker_skipped = marker_skipped.saturating_add(1);
                }
                if let Some(path) = path {
                    scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color,
                    });
                }
            }
        }

        (false, marker_skipped)
    }

    pub(super) fn paint_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        geom: &CanvasGeometry,
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
        let mut wire_budget = WorkBudget::new(u32::MAX / 2);

        for edge in edges_normal
            .into_iter()
            .chain(edges_selected)
            .chain(edges_hovered)
        {
            let (_stop, skipped) = self.push_edge_wire_and_markers_budgeted(
                cx.scene,
                cx.services,
                zoom,
                cx.scale_factor,
                edge.route,
                edge.from,
                edge.to,
                edge.color,
                edge.width,
                edge.start_marker.as_ref(),
                edge.end_marker.as_ref(),
                &mut wire_budget,
                &mut marker_budget,
                false,
            );
            marker_budget_skipped = marker_budget_skipped.saturating_add(skipped);
        }

        if marker_budget_skipped > 0 {
            cx.request_redraw();
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

        if render
            .edges
            .iter()
            .any(|e| e.hint.label.as_ref().is_some_and(|s| !s.is_empty()))
        {
            let label_budget_limit =
                Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
            let mut label_budget = WorkBudget::new(label_budget_limit);
            let (next_edge, skipped_by_budget) = self.paint_edge_labels_static_budgeted(
                cx.scene,
                cx.services,
                cx.scale_factor,
                &render.edges,
                zoom,
                0,
                &mut label_budget,
            );
            let mut label_budget_skipped: u32 = 0;
            if skipped_by_budget && next_edge < render.edges.len() {
                label_budget_skipped = 1;
                cx.request_redraw();
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
                .and_then(|port| geom.port_center(port))
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

    pub(super) fn paint_edge_overlays_selected_hovered<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        zoom: f32,
    ) {
        let hovered_edge = self.interaction.hover_edge;
        let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();

        let mut overlay_edges: Vec<EdgeId> = snapshot.selected_edges.clone();
        if let Some(hovered) = hovered_edge
            && !selected_edges.contains(&hovered)
        {
            overlay_edges.push(hovered);
        }

        if overlay_edges.is_empty() {
            return;
        }

        struct OverlayEdgeDraw {
            from: Point,
            to: Point,
            hint: EdgeRenderHint,
            color: Color,
            width: f32,
        }

        let presenter = &self.presenter;
        let edge_types = self.edge_types.as_ref();
        let style = &self.style;

        let mut edges_to_draw: Vec<OverlayEdgeDraw> = Vec::new();
        let _ = self.graph.read_ref(cx.app, |g| {
            for edge_id in &overlay_edges {
                let Some(edge) = g.edges.get(edge_id) else {
                    continue;
                };
                let Some(from) = geom.port_center(edge.from) else {
                    continue;
                };
                let Some(to) = geom.port_center(edge.to) else {
                    continue;
                };

                let base_hint = presenter.edge_render_hint(g, *edge_id, style);
                let hint = if let Some(edge_types) = edge_types {
                    edge_types.apply(g, *edge_id, style, base_hint)
                } else {
                    base_hint
                }
                .normalized();

                let mut color = presenter.edge_color(g, *edge_id, style);
                if let Some(override_color) = hint.color {
                    color = override_color;
                }

                let mut width = style.wire_width * hint.width_mul;
                let is_selected = selected_edges.contains(edge_id);
                let is_hovered = hovered_edge == Some(*edge_id);
                if is_selected {
                    width *= style.wire_width_selected_mul;
                }
                if is_hovered {
                    width *= style.wire_width_hover_mul;
                }

                edges_to_draw.push(OverlayEdgeDraw {
                    from,
                    to,
                    hint,
                    color,
                    width,
                });
            }
            Some(())
        });

        let mut marker_budget = WorkBudget::new(u32::MAX / 2);
        for edge in edges_to_draw {
            if let Some(path) = self.paint_cache.wire_path(
                cx.services,
                edge.hint.route,
                edge.from,
                edge.to,
                zoom,
                cx.scale_factor,
                edge.width,
            ) {
                cx.scene.push(SceneOp::Path {
                    order: DrawOrder(2),
                    origin: Point::new(Px(0.0), Px(0.0)),
                    path,
                    color: edge.color,
                });
            }

            if let Some(marker) = edge.hint.end_marker.as_ref() {
                let (path, _skipped) = self.paint_cache.edge_end_marker_path_budgeted(
                    cx.services,
                    edge.hint.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                    &mut marker_budget,
                );
                if let Some(path) = path {
                    cx.scene.push(SceneOp::Path {
                        order: DrawOrder(2),
                        origin: Point::new(Px(0.0), Px(0.0)),
                        path,
                        color: edge.color,
                    });
                }
            }

            if let Some(marker) = edge.hint.start_marker.as_ref() {
                let (path, _skipped) = self.paint_cache.edge_start_marker_path_budgeted(
                    cx.services,
                    edge.hint.route,
                    edge.from,
                    edge.to,
                    zoom,
                    cx.scale_factor,
                    marker,
                    self.style.pin_radius,
                    &mut marker_budget,
                );
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
    }

    pub(super) fn paint_edge_labels_static_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        edges: &[EdgeRender],
        zoom: f32,
        start_edge: usize,
        budget: &mut WorkBudget,
    ) -> (usize, bool) {
        let pad_screen = 6.0;
        let corner_screen = 8.0;
        let offset_screen = 10.0;

        let mut edge_text_style = self.style.context_menu_text_style.clone();
        edge_text_style.size = Px(edge_text_style.size.0 / zoom);
        if let Some(lh) = edge_text_style.line_height.as_mut() {
            lh.0 /= zoom;
        }

        let mut next_edge = start_edge.min(edges.len());
        for edge in edges.iter().skip(next_edge) {
            next_edge = next_edge.saturating_add(1);

            let Some(label) = edge.hint.label.as_ref().filter(|s| !s.is_empty()) else {
                continue;
            };

            let (pos, normal) = match edge.hint.route {
                EdgeRouteKind::Bezier => {
                    let (c1, c2) = wire_ctrl_points(edge.from, edge.to, zoom);
                    let p = cubic_bezier(edge.from, c1, c2, edge.to, 0.5);
                    let d = cubic_bezier_derivative(edge.from, c1, c2, edge.to, 0.5);
                    (p, normal_from_tangent(d))
                }
                EdgeRouteKind::Straight => {
                    let p = Point::new(
                        Px(0.5 * (edge.from.x.0 + edge.to.x.0)),
                        Px(0.5 * (edge.from.y.0 + edge.to.y.0)),
                    );
                    let d = Point::new(
                        Px(edge.to.x.0 - edge.from.x.0),
                        Px(edge.to.y.0 - edge.from.y.0),
                    );
                    (p, normal_from_tangent(d))
                }
                EdgeRouteKind::Step => {
                    let mx = 0.5 * (edge.from.x.0 + edge.to.x.0);
                    let p = Point::new(Px(mx), Px(0.5 * (edge.from.y.0 + edge.to.y.0)));
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
                scale_factor: scale_factor * zoom,
            };

            let (prepared, skipped_by_budget) = self.paint_cache.text_blob_budgeted(
                services,
                label.clone(),
                &edge_text_style,
                constraints,
                budget,
            );
            if skipped_by_budget {
                return (next_edge.saturating_sub(1), true);
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

            scene.push(SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                background: self.style.context_menu_background,
                border: Edges::all(Px(1.0 / z)),
                border_color: self.style.context_menu_border,
                corner_radii: Corners::all(Px(corner_screen / z)),
            });

            let text_x = Px(rect.origin.x.0 + pad);
            let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
            scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.context_menu_text,
            });
        }

        (next_edge, false)
    }
}
