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

        let geom_key = self.geometry_key(host, snapshot);
        let key = InternalsCacheKey {
            base: geom_key.base,
            view: InternalsViewKey {
                pan_x_bits: snapshot.pan.x.to_bits(),
                pan_y_bits: snapshot.pan.y.to_bits(),
                bounds_x_bits: bounds.origin.x.0.to_bits(),
                bounds_y_bits: bounds.origin.y.0.to_bits(),
                bounds_w_bits: bounds.size.width.0.to_bits(),
                bounds_h_bits: bounds.size.height.0.to_bits(),
            },
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
        let edge_ctx = EdgePathContext::new(&style, presenter, edge_types);
        let edge_centers: Vec<(crate::core::EdgeId, Point)> = self
            .graph
            .read_ref(host, |graph| {
                graph
                    .edges
                    .iter()
                    .filter_map(|(&edge_id, edge)| {
                        let from = geom.port_center(edge.from)?;
                        let to = geom.port_center(edge.to)?;

                        let hint = edge_ctx.edge_render_hint_normalized(graph, edge_id);

                        let center = if edge_ctx.has_custom_paths() {
                            if let Some(custom) =
                                edge_ctx.edge_custom_path(graph, edge_id, &hint, from, to, zoom)
                            {
                                path_midpoint_and_normal(&custom.commands, bezier_steps)
                                    .map(|(p, _n)| p)
                                    .unwrap_or_else(|| {
                                        Self::edge_center_canvas(hint.route, from, to, zoom)
                                    })
                            } else {
                                Self::edge_center_canvas(hint.route, from, to, zoom)
                            }
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
        let from = Point::new(Px(10.0), Px(50.0));
        let to = Point::new(Px(110.0), Px(150.0));
        let got = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::edge_center_canvas(
            EdgeRouteKind::Step,
            from,
            to,
            1.0,
        );
        assert!((got.x.0 - 60.0).abs() <= 1.0e-6);
        assert!((got.y.0 - 100.0).abs() <= 1.0e-6);
    }
}
