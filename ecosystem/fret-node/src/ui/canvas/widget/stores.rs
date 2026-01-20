use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    fn edge_center_canvas(route: EdgeRouteKind, from: Point, to: Point, zoom: f32) -> Point {
        match route {
            EdgeRouteKind::Bezier => {
                let (c1, c2) = wire_ctrl_points(from, to, zoom);
                cubic_bezier(from, c1, c2, to, 0.5)
            }
            EdgeRouteKind::Straight => {
                Point::new(Px(0.5 * (from.x.0 + to.x.0)), Px(0.5 * (from.y.0 + to.y.0)))
            }
            EdgeRouteKind::Step => {
                let mx = 0.5 * (from.x.0 + to.x.0);
                Point::new(Px(mx), Px(0.5 * (from.y.0 + to.y.0)))
            }
        }
    }

    pub(super) fn update_internals_store<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        bounds: Rect,
        geom: &CanvasGeometry,
    ) {
        let Some(store) = self.internals.as_ref() else {
            return;
        };

        let graph_rev = self.graph.revision(host).unwrap_or(0);
        let presenter_rev = self.presenter.geometry_revision();
        let edge_types_rev = self.edge_types.as_ref().map(|t| t.revision()).unwrap_or(0);
        let node_origin = snapshot.interaction.node_origin.normalized();
        let key = InternalsCacheKey {
            graph_rev,
            zoom_bits: snapshot.zoom.to_bits(),
            node_origin_x_bits: node_origin.x.to_bits(),
            node_origin_y_bits: node_origin.y.to_bits(),
            draw_order_hash: Self::draw_order_hash(&snapshot.draw_order),
            presenter_rev,
            edge_types_rev,
            pan_x_bits: snapshot.pan.x.to_bits(),
            pan_y_bits: snapshot.pan.y.to_bits(),
            bounds_x_bits: bounds.origin.x.0.to_bits(),
            bounds_y_bits: bounds.origin.y.0.to_bits(),
            bounds_w_bits: bounds.size.width.0.to_bits(),
            bounds_h_bits: bounds.size.height.0.to_bits(),
        };

        if self.internals_key == Some(key) {
            return;
        }
        self.internals_key = Some(key);

        let transform = NodeGraphCanvasTransform {
            bounds_origin: bounds.origin,
            bounds_size: bounds.size,
            pan: snapshot.pan,
            zoom: snapshot.zoom,
        };

        let mut next = NodeGraphInternalsSnapshot {
            transform,
            ..NodeGraphInternalsSnapshot::default()
        };

        for (&node, node_geom) in &geom.nodes {
            next.nodes_window
                .insert(node, transform.canvas_rect_to_window(node_geom.rect));
        }
        for (&port, handle) in &geom.ports {
            next.ports_window
                .insert(port, transform.canvas_rect_to_window(handle.bounds));
            next.port_centers_window
                .insert(port, transform.canvas_point_to_window(handle.center));
        }

        let style = self.style.clone();
        let zoom = snapshot.zoom;
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let edge_types = self.edge_types.as_ref();
        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let edge_centers: Vec<(crate::core::EdgeId, Point)> = self
            .graph
            .read_ref(host, |graph| {
                graph
                    .edges
                    .iter()
                    .filter_map(|(&edge_id, edge)| {
                        let from = geom.port_center(edge.from)?;
                        let to = geom.port_center(edge.to)?;

                        let base = presenter.edge_render_hint(graph, edge_id, &style);
                        let hint = if let Some(edge_types) = edge_types {
                            edge_types.apply(graph, edge_id, &style, base).normalized()
                        } else {
                            base.normalized()
                        };

                        let center = if let Some(edge_types) = edge_types
                            && edge_types.has_custom_paths()
                            && let Some(custom) = edge_types.custom_path(
                                graph,
                                edge_id,
                                &style,
                                &hint,
                                crate::ui::edge_types::EdgePathInput { from, to, zoom },
                            ) {
                            path_midpoint_and_normal(&custom.commands, bezier_steps)
                                .map(|(p, _n)| p)
                                .unwrap_or_else(|| {
                                    Self::edge_center_canvas(hint.route, from, to, zoom)
                                })
                        } else {
                            Self::edge_center_canvas(hint.route, from, to, zoom)
                        };
                        Some((edge_id, center))
                    })
                    .collect()
            })
            .ok()
            .unwrap_or_default();

        for (edge, center_canvas) in edge_centers {
            next.edge_centers_window
                .insert(edge, transform.canvas_point_to_window(center_canvas));
        }

        next.focused_node = self.interaction.focused_node;
        next.focused_port = self.interaction.focused_port;
        next.focused_edge = self.interaction.focused_edge;
        next.connecting = self.interaction.wire_drag.is_some();

        let focused_node = self.interaction.focused_node;
        let focused_port = self.interaction.focused_port;
        let focused_edge = self.interaction.focused_edge;
        let labels = self
            .graph
            .read_ref(host, |graph| {
                let node_label = focused_node
                    .and_then(|node| self.presenter.a11y_node_label(graph, node))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_node.map(|node| format!("{:?}", node)));

                let port_label = focused_port
                    .and_then(|port| self.presenter.a11y_port_label(graph, port))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_port.map(|port| format!("{:?}", port)));

                let edge_label = focused_edge
                    .and_then(|edge| self.presenter.a11y_edge_label(graph, edge, &style))
                    .map(|label| format!("{}", label))
                    .or_else(|| focused_edge.map(|edge| format!("{:?}", edge)));

                (node_label, port_label, edge_label)
            })
            .ok()
            .unwrap_or_default();

        next.a11y_focused_node_label = labels.0.clone().map(|label| format!("Node {}", label));
        next.a11y_focused_port_label = labels.1.clone().map(|label| format!("Port {}", label));
        next.a11y_focused_edge_label = labels.2.clone().map(|label| format!("Edge {}", label));
        next.a11y_active_descendant_label = next
            .a11y_focused_port_label
            .clone()
            .or_else(|| next.a11y_focused_edge_label.clone())
            .or_else(|| next.a11y_focused_node_label.clone());

        store.update(next);
    }

    pub(super) fn update_measured_output_store(&mut self, zoom: f32, geom: &CanvasGeometry) {
        let Some(store) = self.measured_output.as_ref() else {
            return;
        };
        let Some(key) = self.geometry.key else {
            return;
        };
        if self.measured_output_key == Some(key) {
            return;
        }
        self.measured_output_key = Some(key);

        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        let quant = |v: f32| {
            (v / crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX).round()
                * crate::ui::measured::MEASURED_GEOMETRY_EPSILON_PX
        };

        let mut node_sizes: Vec<(GraphNodeId, (f32, f32))> = Vec::with_capacity(geom.nodes.len());
        for (&node, node_geom) in &geom.nodes {
            let w = quant(node_geom.rect.size.width.0 * zoom);
            let h = quant(node_geom.rect.size.height.0 * zoom);
            node_sizes.push((node, (w, h)));
        }

        let mut port_anchors: Vec<(PortId, PortAnchorHint)> = Vec::with_capacity(geom.ports.len());
        for (&port, handle) in &geom.ports {
            let Some(node_geom) = geom.nodes.get(&handle.node) else {
                continue;
            };
            let ox = node_geom.rect.origin.x.0;
            let oy = node_geom.rect.origin.y.0;

            let cx = quant((handle.center.x.0 - ox) * zoom);
            let cy = quant((handle.center.y.0 - oy) * zoom);
            let bx = quant((handle.bounds.origin.x.0 - ox) * zoom);
            let by = quant((handle.bounds.origin.y.0 - oy) * zoom);
            let bw = quant(handle.bounds.size.width.0 * zoom);
            let bh = quant(handle.bounds.size.height.0 * zoom);

            let center = Point::new(Px(cx), Px(cy));
            let bounds = Rect::new(Point::new(Px(bx), Px(by)), Size::new(Px(bw), Px(bh)));
            port_anchors.push((port, PortAnchorHint { center, bounds }));
        }

        let _ = store.apply_exclusive_batch_if_changed(
            crate::ui::measured::MeasuredGeometryExclusiveBatch {
                node_sizes_px: node_sizes,
                port_anchors_px: port_anchors,
            },
            crate::ui::measured::MeasuredGeometryApplyOptions::default(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_center_canvas_matches_bezier_math() {
        let from = Point::new(Px(0.0), Px(0.0));
        let to = Point::new(Px(100.0), Px(0.0));
        let zoom = 1.0;
        let (c1, c2) = wire_ctrl_points(from, to, zoom);
        let expected = cubic_bezier(from, c1, c2, to, 0.5);
        let got = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_center_canvas(
            EdgeRouteKind::Bezier,
            from,
            to,
            zoom,
        );
        assert!((got.x.0 - expected.x.0).abs() <= 1.0e-6);
        assert!((got.y.0 - expected.y.0).abs() <= 1.0e-6);
    }

    #[test]
    fn edge_center_canvas_step_uses_mid_x_and_mid_y() {
        let from = Point::new(Px(10.0), Px(20.0));
        let to = Point::new(Px(30.0), Px(60.0));
        let got = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_center_canvas(
            EdgeRouteKind::Step,
            from,
            to,
            1.0,
        );
        assert_eq!(got.x.0, 20.0);
        assert_eq!(got.y.0, 40.0);
    }
}
