use super::super::paint_render_data::RenderData;
use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges<H: UiHost>(
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
            id: EdgeId,
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
        let elevate_edges_on_select = snapshot.interaction.elevate_edges_on_select;

        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let custom_paths = self.collect_custom_edge_paths(&*cx.app, &render.edges, zoom);
        let edge_insert_marker: Option<(Point, Color)> =
            edge_insert_marker_request.and_then(|(edge_id, pos)| {
                render.edges.iter().find(|e| e.id == edge_id).map(|e| {
                    (
                        custom_paths
                            .get(&edge_id)
                            .map(|custom| {
                                closest_point_on_path(&custom.commands, bezier_steps, pos)
                            })
                            .unwrap_or_else(|| {
                                closest_point_on_edge_route(
                                    e.hint.route,
                                    e.from,
                                    e.to,
                                    zoom,
                                    bezier_steps,
                                    pos,
                                )
                            }),
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
                        custom_paths
                            .get(&edge_id)
                            .map(|custom| {
                                closest_point_on_path(&custom.commands, bezier_steps, p.pos)
                            })
                            .unwrap_or_else(|| {
                                closest_point_on_edge_route(
                                    edge.hint.route,
                                    edge.from,
                                    edge.to,
                                    zoom,
                                    bezier_steps,
                                    p.pos,
                                )
                            }),
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
                id: edge.id,
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
            } else if edge.selected && elevate_edges_on_select {
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
            let (_stop, skipped) = if let Some(custom) = custom_paths.get(&edge.id) {
                let fallback = Point::new(
                    Px(edge.to.x.0 - edge.from.x.0),
                    Px(edge.to.y.0 - edge.from.y.0),
                );
                let (t0, t1) =
                    path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
                self.push_edge_custom_wire_and_markers_budgeted(
                    cx.scene,
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
                    edge.width,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut wire_budget,
                    &mut marker_budget,
                    false,
                )
            } else {
                self.push_edge_wire_and_markers_budgeted(
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
                )
            };
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
                (!custom_paths.is_empty()).then_some(&custom_paths),
                bezier_steps,
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
}
