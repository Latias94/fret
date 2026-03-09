use super::*;
use fret_core::scene::PaintBindingV1;

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
                let label_overhead = this.node_render_label_overhead();

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
                    let (node_candidates, visible_nodes) =
                        this.visible_node_ids_for_render(geom, index, cull);
                    out.metrics.node_candidates = node_candidates;

                    for node in visible_nodes {
                        this.append_node_render_data(
                            graph,
                            geom,
                            presenter,
                            &mut out,
                            node,
                            selected.contains(&node),
                            zoom,
                            label_overhead,
                        );
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

                        let selected = selected_edges.contains(&edge_id);
                        let hovered = hovered_edge == Some(edge_id);
                        let hint = if let Some(skin) = this.skin.as_ref() {
                            skin.edge_render_hint(
                                graph,
                                edge_id,
                                &this.style,
                                &hint,
                                selected,
                                hovered,
                            )
                            .normalized()
                        } else {
                            hint
                        };

                        let mut hint = hint;
                        let paint_override = this
                            .paint_overrides
                            .as_ref()
                            .and_then(|o| o.edge_paint_override(edge_id));
                        if let Some(ov) = paint_override {
                            if let Some(dash) = ov.dash {
                                hint.dash = Some(dash);
                            }
                            if let Some(w) = ov.stroke_width_mul {
                                hint.width_mul = hint.width_mul * w;
                            }
                        }
                        hint = hint.normalized();

                        if let Some(c) = cull {
                            let interaction_width_px = this
                                .geometry_overrides
                                .as_ref()
                                .and_then(|o| {
                                    o.edge_geometry_override(edge_id).interaction_width_px
                                })
                                .unwrap_or(snapshot.interaction.edge_interaction_width);
                            let pad = (interaction_width_px
                                .max(
                                    this.style.geometry.wire_width
                                        * hint.width_mul
                                        * this.style.paint.wire_width_selected_mul,
                                )
                                .max(
                                    this.style.geometry.wire_width
                                        * hint.width_mul
                                        * this.style.paint.wire_width_hover_mul,
                                ))
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

                        let mut paint: PaintBindingV1 = color.into();
                        if let Some(ov) = paint_override {
                            if let Some(p) = ov.stroke_paint {
                                paint = p;
                            }
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
                        out.edges.push(EdgeRender {
                            id: edge_id,
                            rank: edge_rank,
                            from,
                            to,
                            color,
                            paint,
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
