use super::super::*;
use crate::ui::canvas::geometry::node_rect_origin_from_anchor;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn drag_preview_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        kind: DragPreviewKind,
        preview_rev: u64,
        nodes: &[(GraphNodeId, CanvasPoint)],
    ) -> Option<(Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>)> {
        if nodes.is_empty() {
            return None;
        }

        let node_origin = snapshot.interaction.node_origin.normalized();

        let base_index_key = self.geometry.index_key?;
        let base_geom = self.geometry.geom.clone();
        let base_index = self.geometry.index.clone();

        let rebuild = self
            .geometry
            .drag_preview_rebuild_needed(kind, base_index_key);
        if rebuild {
            let node_ports = self
                .graph
                .read_ref(host, |g| {
                    let mut out: HashMap<GraphNodeId, Vec<PortId>> = HashMap::new();
                    for (id, _pos) in nodes {
                        let Some(node) = g.nodes.get(id) else {
                            continue;
                        };
                        out.insert(*id, node.ports.clone());
                    }
                    out
                })
                .ok()
                .unwrap_or_default();

            let mut geom = (*base_geom).clone();
            let mut index = (*base_index).clone();
            let mut node_positions: HashMap<GraphNodeId, CanvasPoint> = HashMap::new();
            let mut node_rects: HashMap<GraphNodeId, Rect> = HashMap::new();

            for (id, pos) in nodes {
                node_positions.insert(*id, *pos);
                let Some(node_geom) = geom.nodes.get_mut(id) else {
                    continue;
                };

                let size_canvas = crate::core::CanvasSize {
                    width: node_geom.rect.size.width.0,
                    height: node_geom.rect.size.height.0,
                };
                let next_origin = node_rect_origin_from_anchor(*pos, size_canvas, node_origin);

                let base_x = node_geom.rect.origin.x.0;
                let base_y = node_geom.rect.origin.y.0;
                let dx = next_origin.x - base_x;
                let dy = next_origin.y - base_y;
                if !dx.is_finite() || !dy.is_finite() {
                    continue;
                }

                node_geom.rect = Rect::new(
                    Point::new(Px(next_origin.x), Px(next_origin.y)),
                    node_geom.rect.size,
                );
                index.update_node_rect(*id, node_geom.rect);
                node_rects.insert(*id, node_geom.rect);

                let Some(ports) = node_ports.get(id) else {
                    continue;
                };
                for port_id in ports {
                    let Some(handle) = geom.ports.get_mut(port_id) else {
                        continue;
                    };
                    handle.center =
                        Point::new(Px(handle.center.x.0 + dx), Px(handle.center.y.0 + dy));
                    handle.bounds = Rect::new(
                        Point::new(
                            Px(handle.bounds.origin.x.0 + dx),
                            Px(handle.bounds.origin.y.0 + dy),
                        ),
                        handle.bounds.size,
                    );
                    index.update_port_rect(*port_id, handle.bounds);
                }
            }

            let mut affected_ports: Vec<PortId> = Vec::new();
            for (id, _pos) in nodes {
                if let Some(ports) = node_ports.get(id) {
                    affected_ports.extend(ports.iter().copied());
                }
            }

            let graph_model = self.graph.clone();
            Self::update_edges_for_ports(
                &mut geom,
                &mut index,
                snapshot.zoom,
                &affected_ports,
                |edge_ids| {
                    graph_model
                        .read_ref(host, |g| {
                            edge_ids
                                .iter()
                                .filter_map(|edge_id| {
                                    g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to))
                                })
                                .collect::<Vec<_>>()
                        })
                        .ok()
                        .unwrap_or_default()
                },
            );

            self.geometry.set_drag_preview(DragPreviewCache {
                kind,
                base_index_key,
                preview_rev,
                geom: Arc::new(geom),
                index: Arc::new(index),
                node_positions,
                node_rects,
                node_ports,
            });
        }

        let graph_model = self.graph.clone();
        self.geometry
            .drag_preview_outputs_for_rev(preview_rev, |meta, geom_mut, index_mut| {
                let mut moved_nodes: Vec<(GraphNodeId, CanvasPoint, CanvasPoint)> = Vec::new();
                let mut next_positions: HashMap<GraphNodeId, CanvasPoint> =
                    meta.node_positions.clone();

                for (id, pos) in nodes {
                    let prev = meta.node_positions.get(id).copied().unwrap_or_default();
                    if prev != *pos {
                        moved_nodes.push((*id, prev, *pos));
                        next_positions.insert(*id, *pos);
                    }
                }

                if !moved_nodes.is_empty() {
                    for (id, prev, next) in &moved_nodes {
                        let Some(node_geom) = geom_mut.nodes.get(id) else {
                            continue;
                        };
                        let size_canvas = crate::core::CanvasSize {
                            width: node_geom.rect.size.width.0,
                            height: node_geom.rect.size.height.0,
                        };
                        let prev_origin =
                            node_rect_origin_from_anchor(*prev, size_canvas, node_origin);
                        let next_origin =
                            node_rect_origin_from_anchor(*next, size_canvas, node_origin);

                        let dx = next_origin.x - prev_origin.x;
                        let dy = next_origin.y - prev_origin.y;
                        if !dx.is_finite() || !dy.is_finite() {
                            continue;
                        }

                        if let Some(node_geom) = geom_mut.nodes.get_mut(id) {
                            node_geom.rect = Rect::new(
                                Point::new(Px(next_origin.x), Px(next_origin.y)),
                                node_geom.rect.size,
                            );
                            index_mut.update_node_rect(*id, node_geom.rect);
                            meta.node_rects.insert(*id, node_geom.rect);
                        }

                        if let Some(ports) = meta.node_ports.get(id) {
                            for port_id in ports {
                                let Some(handle) = geom_mut.ports.get_mut(port_id) else {
                                    continue;
                                };
                                handle.center = Point::new(
                                    Px(handle.center.x.0 + dx),
                                    Px(handle.center.y.0 + dy),
                                );
                                handle.bounds = Rect::new(
                                    Point::new(
                                        Px(handle.bounds.origin.x.0 + dx),
                                        Px(handle.bounds.origin.y.0 + dy),
                                    ),
                                    handle.bounds.size,
                                );
                                index_mut.update_port_rect(*port_id, handle.bounds);
                            }
                        }
                    }

                    let mut affected_ports: Vec<PortId> = Vec::new();
                    for (id, _prev, _next) in &moved_nodes {
                        if let Some(ports) = meta.node_ports.get(id) {
                            affected_ports.extend(ports.iter().copied());
                        }
                    }

                    Self::update_edges_for_ports(
                        geom_mut,
                        index_mut,
                        snapshot.zoom,
                        &affected_ports,
                        |edge_ids| {
                            super::resolve_edge_endpoints_from_model(host, &graph_model, edge_ids)
                        },
                    );

                    *meta.node_positions = next_positions;
                }
            })
    }
}
