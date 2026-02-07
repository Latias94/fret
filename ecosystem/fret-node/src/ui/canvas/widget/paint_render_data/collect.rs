use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_render_data<H: UiHost>(
        &self,
        host: &H,
        snapshot: &ViewSnapshot,
        geom: Arc<CanvasGeometry>,
        index: Arc<CanvasSpatialDerived>,
        render_cull_rect: Option<Rect>,
        zoom: f32,
        hovered_edge: Option<EdgeId>,
        include_groups: bool,
        include_nodes: bool,
        include_edges: bool,
    ) -> RenderData {
        let selected: HashSet<GraphNodeId> = snapshot.selected_nodes.iter().copied().collect();
        let selected_edges: HashSet<EdgeId> = snapshot.selected_edges.iter().copied().collect();
        let selected_groups: HashSet<crate::core::GroupId> =
            snapshot.selected_groups.iter().copied().collect();

        let presenter: &dyn NodeGraphPresenter = &*self.presenter;
        let cull = render_cull_rect;
        let this = self;

        this.graph
            .read_ref(host, |graph| {
                let mut out = RenderData::default();

                let geom = geom.as_ref();
                let index = index.as_ref();
                let node_pad = this.style.node_padding;
                let pin_gap = 8.0;
                let pin_r = this.style.pin_radius;
                let label_overhead = 2.0 * node_pad + 2.0 * (pin_r + pin_gap);

                if include_groups {
                    let order = group_order(graph, &snapshot.group_draw_order);
                    out.metrics.group_total = order.len();
                    for group_id in order {
                        out.metrics.group_candidates =
                            out.metrics.group_candidates.saturating_add(1);
                        let Some(group) = graph.groups.get(&group_id) else {
                            continue;
                        };
                        let rect0 = this.group_rect_with_preview(group_id, group.rect);
                        let rect = Rect::new(
                            Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                            Size::new(Px(rect0.size.width), Px(rect0.size.height)),
                        );
                        if cull.is_some_and(|c| !rects_intersect(rect, c)) {
                            continue;
                        }
                        out.groups.push((
                            rect,
                            Arc::<str>::from(group.title.clone()),
                            selected_groups.contains(&group_id),
                        ));
                        out.metrics.group_visible = out.metrics.group_visible.saturating_add(1);
                    }
                }

                if include_nodes {
                    out.metrics.node_total = geom.order.len();
                    if let Some(c) = cull {
                        let mut candidates: Vec<GraphNodeId> = Vec::new();
                        index.query_nodes_in_rect(c, &mut candidates);
                        out.metrics.node_candidates = candidates.len();

                        let mut visible: Vec<GraphNodeId> = Vec::with_capacity(candidates.len());
                        for node in candidates {
                            let Some(node_geom) = geom.nodes.get(&node) else {
                                continue;
                            };
                            if rects_intersect(node_geom.rect, c) {
                                visible.push(node);
                            }
                        }

                        visible.sort_unstable_by_key(|node| {
                            (geom.node_rank.get(node).copied().unwrap_or(u32::MAX), *node)
                        });

                        for node in visible {
                            let Some(node_geom) = geom.nodes.get(&node) else {
                                continue;
                            };
                            let is_selected = selected.contains(&node);
                            let title = presenter.node_title(graph, node);
                            let (inputs, outputs) = node_ports(graph, node);
                            let pin_rows = inputs.len().max(outputs.len());
                            let body = presenter.node_body_label(graph, node);
                            let resize_handles =
                                presenter.node_resize_handles(graph, node, &this.style);
                            out.nodes.push((
                                node,
                                node_geom.rect,
                                is_selected,
                                title,
                                body,
                                pin_rows,
                                resize_handles,
                            ));
                            out.metrics.node_visible = out.metrics.node_visible.saturating_add(1);

                            // Only build port labels/pins for visible nodes (but keep edge endpoints
                            // available via `CanvasGeometry` lookups).
                            let screen_w = node_geom.rect.size.width.0 * zoom;
                            let screen_max = (screen_w - label_overhead).max(0.0);
                            let max_w = Px(screen_max / zoom);

                            for port_id in inputs.iter().chain(outputs.iter()).copied() {
                                let Some(handle) = geom.ports.get(&port_id) else {
                                    continue;
                                };
                                out.port_centers.insert(port_id, handle.center);
                                out.port_labels.insert(
                                    port_id,
                                    PortLabelRender {
                                        label: presenter.port_label(graph, port_id),
                                        dir: handle.dir,
                                        max_width: max_w,
                                    },
                                );
                                let color = presenter.port_color(graph, port_id, &this.style);
                                out.pins.push((port_id, handle.bounds, color));
                            }
                        }
                    } else {
                        out.metrics.node_candidates = geom.order.len();
                        for node in geom.order.iter().copied() {
                            let Some(node_geom) = geom.nodes.get(&node) else {
                                continue;
                            };
                            let is_selected = selected.contains(&node);
                            let title = presenter.node_title(graph, node);
                            let (inputs, outputs) = node_ports(graph, node);
                            let pin_rows = inputs.len().max(outputs.len());
                            let body = presenter.node_body_label(graph, node);
                            let resize_handles =
                                presenter.node_resize_handles(graph, node, &this.style);
                            out.nodes.push((
                                node,
                                node_geom.rect,
                                is_selected,
                                title,
                                body,
                                pin_rows,
                                resize_handles,
                            ));
                            out.metrics.node_visible = out.metrics.node_visible.saturating_add(1);

                            // Only build port labels/pins for visible nodes (but keep edge endpoints
                            // available via `CanvasGeometry` lookups).
                            let screen_w = node_geom.rect.size.width.0 * zoom;
                            let screen_max = (screen_w - label_overhead).max(0.0);
                            let max_w = Px(screen_max / zoom);

                            for port_id in inputs.iter().chain(outputs.iter()).copied() {
                                let Some(handle) = geom.ports.get(&port_id) else {
                                    continue;
                                };
                                out.port_centers.insert(port_id, handle.center);
                                out.port_labels.insert(
                                    port_id,
                                    PortLabelRender {
                                        label: presenter.port_label(graph, port_id),
                                        dir: handle.dir,
                                        max_width: max_w,
                                    },
                                );
                                let color = presenter.port_color(graph, port_id, &this.style);
                                out.pins.push((port_id, handle.bounds, color));
                            }
                        }
                    }
                }

                if include_edges {
                    out.metrics.edge_total = graph.edges.len();
                    let mut edge_ids: Vec<EdgeId> = Vec::new();
                    if let Some(c) = cull {
                        index.query_edges_in_rect(c, &mut edge_ids);
                    } else {
                        edge_ids.extend(graph.edges.keys().copied());
                    }
                    out.metrics.edge_candidates = edge_ids.len();

                    for edge_id in edge_ids {
                        let Some(edge) = graph.edges.get(&edge_id) else {
                            continue;
                        };
                        if this
                            .interaction
                            .wire_drag
                            .as_ref()
                            .is_some_and(|w| Self::wire_drag_suppresses_edge(&w.kind, edge_id))
                        {
                            continue;
                        }
                        use std::collections::hash_map::Entry;

                        let from = match out.port_centers.entry(edge.from) {
                            Entry::Occupied(v) => *v.get(),
                            Entry::Vacant(v) => {
                                let Some(center) = geom.port_center(edge.from) else {
                                    continue;
                                };
                                *v.insert(center)
                            }
                        };
                        let to = match out.port_centers.entry(edge.to) {
                            Entry::Occupied(v) => *v.get(),
                            Entry::Vacant(v) => {
                                let Some(center) = geom.port_center(edge.to) else {
                                    continue;
                                };
                                *v.insert(center)
                            }
                        };
                        let hint = EdgePathContext::new(
                            &this.style,
                            &*this.presenter,
                            this.edge_types.as_ref(),
                        )
                        .edge_render_hint_normalized(graph, edge_id);
                        if let Some(c) = cull {
                            let pad = (snapshot
                                .interaction
                                .edge_interaction_width
                                .max(this.style.wire_width * this.style.wire_width_selected_mul)
                                .max(this.style.wire_width * this.style.wire_width_hover_mul))
                                / zoom;
                            let bounds = if let Some(custom) =
                                this.edge_custom_path(graph, edge_id, &hint, from, to, zoom)
                            {
                                path_bounds_rect(&custom.commands)
                                    .map(|r| inflate_rect(r, pad))
                                    .unwrap_or_else(|| {
                                        edge_bounds_rect(hint.route, from, to, zoom, pad)
                                    })
                            } else {
                                edge_bounds_rect(hint.route, from, to, zoom, pad)
                            };
                            if !rects_intersect(bounds, c) {
                                continue;
                            }
                        }
                        let mut color = presenter.edge_color(graph, edge_id, &this.style);
                        if let Some(override_color) = hint.color {
                            color = override_color;
                        }

                        // Keep edge ordering stable and aligned with node z-order.
                        // This mirrors XyFlow's "basic" z-index behavior where edge z depends on the
                        // highest z of its endpoints (selection/hover boosts are applied later by
                        // drawing those edges last).
                        let edge_rank = geom
                            .ports
                            .get(&edge.from)
                            .and_then(|p| geom.node_rank.get(&p.node).copied())
                            .unwrap_or(0)
                            .max(
                                geom.ports
                                    .get(&edge.to)
                                    .and_then(|p| geom.node_rank.get(&p.node).copied())
                                    .unwrap_or(0),
                            );
                        let selected = selected_edges.contains(&edge_id);
                        let hovered = hovered_edge == Some(edge_id);
                        out.edges.push(EdgeRender {
                            id: edge_id,
                            rank: edge_rank,
                            from,
                            to,
                            color,
                            hint,
                            selected,
                            hovered,
                        });
                        out.metrics.edge_visible = out.metrics.edge_visible.saturating_add(1);
                    }

                    out.edges
                        .sort_unstable_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.id.cmp(&b.id)));
                }

                out
            })
            .unwrap_or_default()
    }
}
