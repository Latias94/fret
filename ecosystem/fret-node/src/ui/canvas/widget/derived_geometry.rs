use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn group_rect_with_preview(
        &self,
        group_id: crate::core::GroupId,
        base: crate::core::CanvasRect,
    ) -> crate::core::CanvasRect {
        if let Some(resize) = self
            .interaction
            .group_resize
            .as_ref()
            .filter(|r| r.group == group_id)
        {
            return resize.current_rect;
        }
        if let Some(drag) = self
            .interaction
            .group_drag
            .as_ref()
            .filter(|d| d.group == group_id)
        {
            return drag.current_rect;
        }
        if let Some(rect) = self.interaction.node_resize.as_ref().and_then(|r| {
            r.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, rect)| *rect)
        }) {
            return rect;
        }
        if let Some(rect) = self.interaction.node_drag.as_ref().and_then(|d| {
            d.current_groups
                .iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, r)| *r)
        }) {
            return rect;
        }
        base
    }

    pub(super) fn canvas_geometry<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> Arc<CanvasGeometry> {
        let zoom = snapshot.zoom;
        let node_origin = snapshot.interaction.node_origin.normalized();
        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let edge_types_rev = self.edge_types.as_ref().map(|t| t.revision()).unwrap_or(0);
        let key = GeometryCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            node_origin_x_bits: node_origin.x.to_bits(),
            node_origin_y_bits: node_origin.y.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
            edge_types_rev,
        };

        if self.geometry.key != Some(key) {
            self.geometry.drag_preview = None;
            let style = self.style.clone();
            let draw_order = snapshot.draw_order.clone();
            let graph = self.graph.clone();
            let presenter = &mut *self.presenter;
            let edge_types = self.edge_types.as_ref();
            let (geom, index) = graph
                .read_ref(host, |graph| {
                    let geom = CanvasGeometry::build_with_presenter(
                        graph,
                        &draw_order,
                        &style,
                        zoom,
                        node_origin,
                        presenter,
                    );
                    let z = zoom.max(1.0e-6);
                    let tuning = snapshot.interaction.spatial_index;
                    let cell_size_canvas = (tuning.cell_size_screen_px / z)
                        .max(tuning.min_cell_size_screen_px / z)
                        .max(1.0);
                    let max_hit_pad_canvas = (tuning.edge_aabb_pad_screen_px / z).max(0.0);
                    let mut index = CanvasSpatialIndex::build(
                        graph,
                        &geom,
                        zoom,
                        max_hit_pad_canvas,
                        cell_size_canvas,
                    );

                    // Stage 2 `edgeTypes`: custom edge paths may exceed the default conservative
                    // wire AABB, so patch the index with a custom bounds rect when available.
                    if let Some(edge_types) = edge_types
                        && edge_types.has_custom_paths()
                    {
                        let pin_pad = (style.pin_radius.max(0.0) / z).max(0.0);
                        for (&edge_id, edge) in &graph.edges {
                            let Some(from) = geom.port_center(edge.from) else {
                                continue;
                            };
                            let Some(to) = geom.port_center(edge.to) else {
                                continue;
                            };

                            let base = presenter.edge_render_hint(graph, edge_id, &style);
                            let hint = edge_types.apply(graph, edge_id, &style, base).normalized();
                            let marker_pad = hint
                                .start_marker
                                .as_ref()
                                .map(|m| (m.size.max(0.0) / z).max(0.0))
                                .unwrap_or(0.0)
                                .max(
                                    hint.end_marker
                                        .as_ref()
                                        .map(|m| (m.size.max(0.0) / z).max(0.0))
                                        .unwrap_or(0.0),
                                );
                            let pad = max_hit_pad_canvas.max(pin_pad).max(marker_pad);

                            let Some(custom) = edge_types.custom_path(
                                graph,
                                edge_id,
                                &style,
                                &hint,
                                crate::ui::edge_types::EdgePathInput { from, to, zoom },
                            ) else {
                                continue;
                            };

                            let Some(bounds) = path_bounds_rect(&custom.commands) else {
                                continue;
                            };
                            index.update_edge_rect(edge_id, inflate_rect(bounds, pad));
                        }
                    }
                    (geom, index)
                })
                .ok()
                .unwrap_or_else(|| (CanvasGeometry::default(), CanvasSpatialIndex::empty()));
            self.geometry.key = Some(key);
            self.geometry.geom = Arc::new(geom);
            self.geometry.index = Arc::new(index);
        }

        self.geometry.geom.clone()
    }

    pub(super) fn edge_focus_anchor_rect(center: Point, zoom: f32) -> Rect {
        let z = zoom.max(1.0e-6);
        let half = 0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN / z;
        let pad = Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN / z;
        let size = 2.0 * (half + pad);
        Rect::new(
            Point::new(Px(center.x.0 - half - pad), Px(center.y.0 - half - pad)),
            Size::new(Px(size), Px(size)),
        )
    }

    pub(super) fn edge_focus_anchor_centers(
        route: EdgeRouteKind,
        from: Point,
        to: Point,
        zoom: f32,
    ) -> (Point, Point) {
        fn norm_or_fallback(v: Point, fallback: Point) -> Point {
            let len = (v.x.0 * v.x.0 + v.y.0 * v.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(v.x.0 / len), Px(v.y.0 / len));
            }
            let len = (fallback.x.0 * fallback.x.0 + fallback.y.0 * fallback.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(fallback.x.0 / len), Px(fallback.y.0 / len));
            }
            Point::new(Px(1.0), Px(0.0))
        }

        let z = zoom.max(1.0e-6);
        let off = Self::EDGE_FOCUS_ANCHOR_OFFSET_SCREEN / z;
        let fallback = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));

        let start_dir = norm_or_fallback(edge_route_start_tangent(route, from, to, zoom), fallback);
        let end_dir = norm_or_fallback(edge_route_end_tangent(route, from, to, zoom), fallback);

        let start = Point::new(
            Px(from.x.0 + start_dir.x.0 * off),
            Px(from.y.0 + start_dir.y.0 * off),
        );
        let end = Point::new(
            Px(to.x.0 - end_dir.x.0 * off),
            Px(to.y.0 - end_dir.y.0 * off),
        );
        (start, end)
    }

    pub(super) fn edge_focus_anchor_centers_from_tangents(
        from: Point,
        to: Point,
        zoom: f32,
        start_tangent: Point,
        end_tangent: Point,
    ) -> (Point, Point) {
        fn norm_or_fallback(v: Point, fallback: Point) -> Point {
            let len = (v.x.0 * v.x.0 + v.y.0 * v.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(v.x.0 / len), Px(v.y.0 / len));
            }
            let len = (fallback.x.0 * fallback.x.0 + fallback.y.0 * fallback.y.0).sqrt();
            if len.is_finite() && len > 1.0e-6 {
                return Point::new(Px(fallback.x.0 / len), Px(fallback.y.0 / len));
            }
            Point::new(Px(1.0), Px(0.0))
        }

        let z = zoom.max(1.0e-6);
        let off = Self::EDGE_FOCUS_ANCHOR_OFFSET_SCREEN / z;
        let fallback = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));

        let start_dir = norm_or_fallback(start_tangent, fallback);
        let end_dir = norm_or_fallback(end_tangent, fallback);

        let start = Point::new(
            Px(from.x.0 + start_dir.x.0 * off),
            Px(from.y.0 + start_dir.y.0 * off),
        );
        let end = Point::new(
            Px(to.x.0 - end_dir.x.0 * off),
            Px(to.y.0 - end_dir.y.0 * off),
        );
        (start, end)
    }

    pub(super) fn canvas_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
    ) -> (Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>) {
        let geom = self.canvas_geometry(host, snapshot);
        let index = self.geometry.index.clone();
        let node_drag = self.interaction.node_drag.clone();
        let group_drag = self.interaction.group_drag.clone();
        let node_resize = self.interaction.node_resize.clone();

        if let Some(drag) = node_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::NodeDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(drag) = group_drag.as_ref() {
            if let Some((geom, index)) = self.drag_preview_derived(
                host,
                snapshot,
                DragPreviewKind::GroupDrag,
                drag.preview_rev,
                &drag.current_nodes,
            ) {
                return (geom, index);
            }
        } else if let Some(resize) = node_resize.as_ref() {
            if let Some((geom, index)) = self.node_resize_preview_derived(
                host,
                snapshot,
                resize.preview_rev,
                resize.node,
                resize.current_node_pos,
                resize.current_size_opt,
            ) {
                return (geom, index);
            }
        } else {
            self.geometry.drag_preview = None;
        }

        (geom, index)
    }

    pub(super) fn update_ports_for_node_rect_change(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        node_id: GraphNodeId,
        prev_rect: Rect,
        next_rect: Rect,
        ports: &[PortId],
    ) {
        let eps = 1.0e-3;
        let prev_w = prev_rect.size.width.0;
        let next_w = next_rect.size.width.0;

        for port_id in ports {
            let Some(handle) = geom.ports.get_mut(port_id) else {
                continue;
            };
            if handle.node != node_id {
                continue;
            }

            let local_x = handle.center.x.0 - prev_rect.origin.x.0;
            let local_y = handle.center.y.0 - prev_rect.origin.y.0;
            let mut next_local_x = local_x;
            match handle.dir {
                PortDirection::In => {
                    if (local_x - 0.0).abs() <= eps {
                        next_local_x = 0.0;
                    }
                }
                PortDirection::Out => {
                    if (local_x - prev_w).abs() <= eps {
                        next_local_x = next_w;
                    }
                }
            }

            let center = Point::new(
                Px(next_rect.origin.x.0 + next_local_x),
                Px(next_rect.origin.y.0 + local_y),
            );
            let half_w = 0.5 * handle.bounds.size.width.0;
            let half_h = 0.5 * handle.bounds.size.height.0;
            let bounds = Rect::new(
                Point::new(Px(center.x.0 - half_w), Px(center.y.0 - half_h)),
                handle.bounds.size,
            );
            handle.center = center;
            handle.bounds = bounds;
            index.update_port_rect(*port_id, bounds);
        }
    }

    pub(super) fn update_edges_for_ports(
        geom: &mut CanvasGeometry,
        index: &mut CanvasSpatialIndex,
        zoom: f32,
        ports: &[PortId],
        resolve_edges: impl FnOnce(&HashSet<EdgeId>) -> Vec<(EdgeId, PortId, PortId)>,
    ) {
        let mut edge_ids: HashSet<EdgeId> = HashSet::new();
        for port in ports {
            if let Some(edges) = index.edges_for_port(*port) {
                edge_ids.extend(edges.iter().copied());
            }
        }
        if edge_ids.is_empty() {
            return;
        }

        let endpoints = resolve_edges(&edge_ids);
        for (edge_id, from, to) in endpoints {
            let Some(p0) = geom.port_center(from) else {
                continue;
            };
            let Some(p1) = geom.port_center(to) else {
                continue;
            };
            let rect = index.edge_aabb(p0, p1, zoom);
            index.update_edge_rect(edge_id, rect);
        }
    }
}
