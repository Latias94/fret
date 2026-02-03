use super::super::geometry::node_rect_origin_from_anchor;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn drag_preview_derived<H: UiHost>(
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
            .drag_preview
            .as_ref()
            .is_none_or(|cache| cache.kind != kind || cache.base_index_key != base_index_key);
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

            let mut affected_edges: Vec<EdgeId> = Vec::new();
            for (id, _pos) in nodes {
                let Some(ports) = node_ports.get(id) else {
                    continue;
                };
                for port in ports {
                    if let Some(edges) = index.edges_for_port(*port) {
                        affected_edges.extend(edges.iter().copied());
                    }
                }
            }
            affected_edges.sort_unstable();
            affected_edges.dedup();

            let edge_endpoints: Vec<(EdgeId, PortId, PortId)> = if affected_edges.is_empty() {
                Vec::new()
            } else {
                self.graph
                    .read_ref(host, |g| {
                        affected_edges
                            .iter()
                            .filter_map(|edge_id| {
                                g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to))
                            })
                            .collect::<Vec<_>>()
                    })
                    .ok()
                    .unwrap_or_default()
            };

            for (edge_id, from, to) in edge_endpoints {
                let Some(p0) = geom.port_center(from) else {
                    continue;
                };
                let Some(p1) = geom.port_center(to) else {
                    continue;
                };
                let rect = index.edge_aabb(p0, p1, snapshot.zoom);
                index.update_edge_rect(edge_id, rect);
            }

            self.geometry.drag_preview = Some(DragPreviewCache {
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

        let Some(cache) = self.geometry.drag_preview.as_mut() else {
            return None;
        };
        if cache.preview_rev != preview_rev {
            let geom_mut = Arc::make_mut(&mut cache.geom);
            let index_mut = Arc::make_mut(&mut cache.index);

            let mut moved_nodes: Vec<(GraphNodeId, CanvasPoint, CanvasPoint)> = Vec::new();
            let mut next_positions: HashMap<GraphNodeId, CanvasPoint> =
                cache.node_positions.clone();

            for (id, pos) in nodes {
                let prev = cache.node_positions.get(id).copied().unwrap_or_default();
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
                    let prev_origin = node_rect_origin_from_anchor(*prev, size_canvas, node_origin);
                    let next_origin = node_rect_origin_from_anchor(*next, size_canvas, node_origin);

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
                        cache.node_rects.insert(*id, node_geom.rect);
                    }

                    if let Some(ports) = cache.node_ports.get(id) {
                        for port_id in ports {
                            let Some(handle) = geom_mut.ports.get_mut(port_id) else {
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
                            index_mut.update_port_rect(*port_id, handle.bounds);
                        }
                    }
                }

                let mut affected_edges: Vec<EdgeId> = Vec::new();
                for (id, _prev, _next) in &moved_nodes {
                    let Some(ports) = cache.node_ports.get(id) else {
                        continue;
                    };
                    for port in ports {
                        if let Some(edges) = index_mut.edges_for_port(*port) {
                            affected_edges.extend(edges.iter().copied());
                        }
                    }
                }
                affected_edges.sort_unstable();
                affected_edges.dedup();

                if !affected_edges.is_empty() {
                    let edge_endpoints: Vec<(EdgeId, PortId, PortId)> = self
                        .graph
                        .read_ref(host, |g| {
                            affected_edges
                                .iter()
                                .filter_map(|edge_id| {
                                    g.edges.get(edge_id).map(|e| (*edge_id, e.from, e.to))
                                })
                                .collect::<Vec<_>>()
                        })
                        .ok()
                        .unwrap_or_default();
                    for (edge_id, from, to) in edge_endpoints {
                        let Some(p0) = geom_mut.port_center(from) else {
                            continue;
                        };
                        let Some(p1) = geom_mut.port_center(to) else {
                            continue;
                        };
                        let rect = index_mut.edge_aabb(p0, p1, snapshot.zoom);
                        index_mut.update_edge_rect(edge_id, rect);
                    }
                }

                cache.node_positions = next_positions;
            }
            cache.preview_rev = preview_rev;
        }

        Some((cache.geom.clone(), cache.index.clone()))
    }

    pub(super) fn node_resize_preview_derived<H: UiHost>(
        &mut self,
        host: &H,
        snapshot: &ViewSnapshot,
        preview_rev: u64,
        node_id: GraphNodeId,
        pos: CanvasPoint,
        size_opt_px: Option<CanvasSize>,
    ) -> Option<(Arc<CanvasGeometry>, Arc<CanvasSpatialIndex>)> {
        let base_index_key = self.geometry.index_key?;
        let base_geom = self.geometry.geom.clone();
        let base_index = self.geometry.index.clone();

        let zoom = snapshot.zoom.max(1.0e-6);
        let base_rect = base_geom.nodes.get(&node_id)?.rect;
        let size = size_opt_px
            .map(|s| {
                Size::new(
                    Px((s.width / zoom).max(0.0)),
                    Px((s.height / zoom).max(0.0)),
                )
            })
            .unwrap_or(base_rect.size);
        let node_origin = snapshot.interaction.node_origin.normalized();
        let size_canvas = crate::core::CanvasSize {
            width: size.width.0,
            height: size.height.0,
        };
        let next_origin = node_rect_origin_from_anchor(pos, size_canvas, node_origin);
        let next_rect = Rect::new(Point::new(Px(next_origin.x), Px(next_origin.y)), size);

        let rebuild = self.geometry.drag_preview.as_ref().is_none_or(|cache| {
            cache.kind != DragPreviewKind::NodeResize || cache.base_index_key != base_index_key
        });
        if rebuild {
            let node_ports = self
                .graph
                .read_ref(host, |g| {
                    let mut out: HashMap<GraphNodeId, Vec<PortId>> = HashMap::new();
                    if let Some(node) = g.nodes.get(&node_id) {
                        out.insert(node_id, node.ports.clone());
                    }
                    out
                })
                .ok()
                .unwrap_or_default();

            let mut geom = (*base_geom).clone();
            let mut index = (*base_index).clone();
            let mut node_positions: HashMap<GraphNodeId, CanvasPoint> = HashMap::new();
            let mut node_rects: HashMap<GraphNodeId, Rect> = HashMap::new();
            node_positions.insert(node_id, pos);
            node_rects.insert(node_id, next_rect);

            if let Some(node_geom) = geom.nodes.get_mut(&node_id) {
                let prev_rect = node_geom.rect;
                if prev_rect != next_rect {
                    node_geom.rect = next_rect;
                    index.update_node_rect(node_id, next_rect);
                    Self::update_ports_for_node_rect_change(
                        &mut geom,
                        &mut index,
                        node_id,
                        prev_rect,
                        next_rect,
                        node_ports
                            .get(&node_id)
                            .map(|v| v.as_slice())
                            .unwrap_or(&[]),
                    );
                }
            }

            Self::update_edges_for_ports(
                &mut geom,
                &mut index,
                snapshot.zoom,
                node_ports
                    .get(&node_id)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]),
                |edge_ids| {
                    self.graph
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

            self.geometry.drag_preview = Some(DragPreviewCache {
                kind: DragPreviewKind::NodeResize,
                base_index_key,
                preview_rev,
                geom: Arc::new(geom),
                index: Arc::new(index),
                node_positions,
                node_rects,
                node_ports,
            });
        }

        let Some(cache) = self.geometry.drag_preview.as_mut() else {
            return None;
        };
        if cache.preview_rev != preview_rev {
            let geom_mut = Arc::make_mut(&mut cache.geom);
            let index_mut = Arc::make_mut(&mut cache.index);

            let prev_rect = cache.node_rects.get(&node_id).copied().unwrap_or(base_rect);
            if prev_rect != next_rect {
                if let Some(node_geom) = geom_mut.nodes.get_mut(&node_id) {
                    node_geom.rect = next_rect;
                    index_mut.update_node_rect(node_id, next_rect);
                    Self::update_ports_for_node_rect_change(
                        geom_mut,
                        index_mut,
                        node_id,
                        prev_rect,
                        next_rect,
                        cache
                            .node_ports
                            .get(&node_id)
                            .map(|v| v.as_slice())
                            .unwrap_or(&[]),
                    );
                }

                Self::update_edges_for_ports(
                    geom_mut,
                    index_mut,
                    snapshot.zoom,
                    cache
                        .node_ports
                        .get(&node_id)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]),
                    |edge_ids| {
                        self.graph
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
            }

            cache.node_positions.insert(node_id, pos);
            cache.node_rects.insert(node_id, next_rect);
            cache.preview_rev = preview_rev;
        }

        Some((cache.geom.clone(), cache.index.clone()))
    }
}
