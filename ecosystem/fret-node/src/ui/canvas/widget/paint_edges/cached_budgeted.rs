use super::super::paint_render_data::EdgeRender;
use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges_cached_budgeted<H: UiHost>(
        &mut self,
        tmp: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        edges: &[EdgeRender],
        zoom: f32,
        scale_factor: f32,
        next_edge: usize,
        wire_budget: &mut WorkBudget,
        marker_budget: &mut WorkBudget,
    ) -> (usize, bool) {
        let custom_paths = self.collect_custom_edge_paths(host, edges, zoom);
        let mut next_edge = next_edge.min(edges.len());

        for edge in edges.iter().skip(next_edge) {
            let width = self.style.wire_width * edge.hint.width_mul.max(0.0);
            let (stop, _marker_skipped) = if let Some(custom) = custom_paths.get(&edge.id) {
                let fallback = Point::new(
                    Px(edge.to.x.0 - edge.from.x.0),
                    Px(edge.to.y.0 - edge.from.y.0),
                );
                let (t0, t1) =
                    path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
                self.push_edge_custom_wire_and_markers_budgeted(
                    tmp,
                    services,
                    custom.cache_key,
                    &custom.commands,
                    t0,
                    t1,
                    zoom,
                    scale_factor,
                    edge.from,
                    edge.to,
                    edge.color,
                    width,
                    edge.hint.start_marker.as_ref(),
                    edge.hint.end_marker.as_ref(),
                    wire_budget,
                    marker_budget,
                    true,
                )
            } else {
                self.push_edge_wire_and_markers_budgeted(
                    tmp,
                    services,
                    zoom,
                    scale_factor,
                    edge.hint.route,
                    edge.from,
                    edge.to,
                    edge.color,
                    width,
                    edge.hint.start_marker.as_ref(),
                    edge.hint.end_marker.as_ref(),
                    wire_budget,
                    marker_budget,
                    true,
                )
            };

            if stop {
                return (next_edge, true);
            }
            next_edge = next_edge.saturating_add(1);
        }

        (next_edge, false)
    }

    pub(in super::super) fn paint_edge_labels_static_budgeted_cached<H: UiHost>(
        &mut self,
        scene: &mut fret_core::Scene,
        host: &H,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        edges: &[EdgeRender],
        bezier_steps: usize,
        zoom: f32,
        start_edge: usize,
        budget: &mut WorkBudget,
    ) -> (usize, bool) {
        let custom_paths = self.collect_custom_edge_paths(host, edges, zoom);
        self.paint_edge_labels_static_budgeted(
            scene,
            services,
            scale_factor,
            edges,
            (!custom_paths.is_empty()).then_some(&custom_paths),
            bezier_steps,
            zoom,
            start_edge,
            budget,
        )
    }

    pub(in super::super) fn paint_edge_labels_static_budgeted(
        &mut self,
        scene: &mut fret_core::Scene,
        services: &mut dyn fret_core::UiServices,
        scale_factor: f32,
        edges: &[EdgeRender],
        custom_paths: Option<&HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>>,
        bezier_steps: usize,
        zoom: f32,
        start_edge: usize,
        budget: &mut WorkBudget,
    ) -> (usize, bool) {
        let pad_screen = self.style.edge_label_padding;
        let corner_screen = self.style.edge_label_corner_radius;
        let offset_screen = self.style.edge_label_offset;
        let max_width_screen = self.style.edge_label_max_width;
        let border_width_screen = self.style.edge_label_border_width;

        let mut edge_text_style = self.style.edge_label_text_style.clone();
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

            let base_pos_normal = || match edge.hint.route {
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

            let (pos, normal) = custom_paths
                .and_then(|m| m.get(&edge.id))
                .and_then(|custom| path_midpoint_and_normal(&custom.commands, bezier_steps))
                .unwrap_or_else(base_pos_normal);

            let z = zoom.max(1.0e-6);
            let off = offset_screen / z;
            let anchor = Point::new(
                Px(pos.x.0 + normal.x.0 * off),
                Px(pos.y.0 + normal.y.0 * off),
            );

            let max_w = max_width_screen.max(0.0) / z;
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

            let pad = pad_screen.max(0.0) / z;
            let w = metrics.size.width.0.max(0.0);
            let h = metrics.size.height.0.max(0.0);
            let rect = Rect::new(
                Point::new(
                    Px(anchor.x.0 - 0.5 * w - pad),
                    Px(anchor.y.0 - 0.5 * h - pad),
                ),
                Size::new(Px(w + 2.0 * pad), Px(h + 2.0 * pad)),
            );

            let border_color = edge.hint.color.unwrap_or(self.style.edge_label_border);
            let border_w = border_width_screen.max(0.0) / z;
            scene.push(SceneOp::Quad {
                order: DrawOrder(2),
                rect,
                background: self.style.edge_label_background,
                border: Edges::all(Px(border_w)),
                border_color,
                corner_radii: Corners::all(Px(corner_screen.max(0.0) / z)),
            });

            let text_x = Px(rect.origin.x.0 + pad);
            let text_y = Px(rect.origin.y.0 + pad + metrics.baseline.0);
            scene.push(SceneOp::Text {
                order: DrawOrder(2),
                origin: Point::new(text_x, text_y),
                text: blob,
                color: self.style.edge_label_text,
            });
        }

        (next_edge, false)
    }
}
