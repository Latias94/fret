use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    // NOTE: Node bounds and port anchors must come from derived geometry (`CanvasGeometry`),
    // not ad-hoc layout guesses. See ADR 0135.

    pub(super) fn rect_contains_point(rect: Rect, pos: Point) -> bool {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);
        pos.x.0 >= min_x && pos.x.0 <= max_x && pos.y.0 >= min_y && pos.y.0 <= max_y
    }

    pub(super) fn distance_sq_point_to_rect(pos: Point, rect: Rect) -> f32 {
        let min_x = rect.origin.x.0.min(rect.origin.x.0 + rect.size.width.0);
        let min_y = rect.origin.y.0.min(rect.origin.y.0 + rect.size.height.0);
        let max_x = rect.origin.x.0.max(rect.origin.x.0 + rect.size.width.0);
        let max_y = rect.origin.y.0.max(rect.origin.y.0 + rect.size.height.0);

        let dx = if pos.x.0 < min_x {
            min_x - pos.x.0
        } else if pos.x.0 > max_x {
            pos.x.0 - max_x
        } else {
            0.0
        };
        let dy = if pos.y.0 < min_y {
            min_y - pos.y.0
        } else if pos.y.0 > max_y {
            pos.y.0 - max_y
        } else {
            0.0
        };

        dx * dx + dy * dy
    }

    pub(super) fn hit_port(
        &self,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<PortId>,
    ) -> Option<PortId> {
        let r = self.style.pin_radius / zoom;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }

        index.query_ports(pos, r, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(PortId, u32)> = None;
        for &port_id in scratch.iter() {
            let Some(handle) = geom.ports.get(&port_id) else {
                continue;
            };
            if !Self::rect_contains_point(handle.bounds, pos) {
                continue;
            }
            let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
            match best {
                Some((best_id, best_rank)) => {
                    if rank > best_rank || (rank == best_rank && port_id < best_id) {
                        best = Some((port_id, rank));
                    }
                }
                None => best = Some((port_id, rank)),
            }
        }

        best.map(|(id, _)| id)
    }

    pub(super) fn pick_target_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        from: PortId,
        require_from_connectable_start: bool,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<PortId>,
    ) -> Option<PortId> {
        if require_from_connectable_start
            && !Self::port_is_connectable_start(graph, &snapshot.interaction, from)
        {
            return None;
        }

        let from_port = graph.ports.get(&from)?;
        let desired_dir = match from_port.dir {
            PortDirection::In => PortDirection::Out,
            PortDirection::Out => PortDirection::In,
        };

        match snapshot.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => {
                let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                let port = graph.ports.get(&candidate)?;
                (candidate != from
                    && port.dir == desired_dir
                    && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                .then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(geom, index, pos, zoom, scratch)?;
                    return (candidate != from
                        && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                    .then_some(candidate);
                }
                let r = canvas_units_from_screen_px(radius_screen, zoom);
                let r2 = r * r;
                let eps = (1.0e-3 / zoom.max(1.0e-6)).max(1.0e-6);

                let mut best: Option<(PortId, f32, bool, u32)> = None;
                index.query_ports(pos, r, scratch);
                scratch.sort_unstable();
                scratch.dedup();
                for &port_id in scratch.iter() {
                    if port_id == from {
                        continue;
                    }
                    let Some(handle) = geom.ports.get(&port_id) else {
                        continue;
                    };
                    if !Self::port_is_connectable_end(graph, &snapshot.interaction, port_id) {
                        continue;
                    }
                    let d2 = Self::distance_sq_point_to_rect(pos, handle.bounds);
                    if d2 > r2 {
                        continue;
                    }
                    let prefers_opposite = handle.dir == desired_dir;
                    let rank = geom.node_rank.get(&handle.node).copied().unwrap_or(0);
                    match best {
                        Some((best_id, best_d2, best_prefers_opposite, best_rank)) => {
                            if d2 + eps < best_d2 {
                                best = Some((port_id, d2, prefers_opposite, rank));
                            } else if (d2 - best_d2).abs() <= eps {
                                if prefers_opposite != best_prefers_opposite {
                                    if prefers_opposite {
                                        best = Some((port_id, d2, prefers_opposite, rank));
                                    }
                                } else if rank > best_rank {
                                    best = Some((port_id, d2, prefers_opposite, rank));
                                } else if rank == best_rank && port_id < best_id {
                                    best = Some((port_id, d2, prefers_opposite, rank));
                                }
                            }
                        }
                        None => best = Some((port_id, d2, prefers_opposite, rank)),
                    }
                }

                best.map(|(id, _, _, _)| id)
            }
        }
    }

    pub(super) fn hit_edge(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<EdgeId>,
    ) -> Option<EdgeId> {
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let hit_w = canvas_units_from_screen_px(snapshot.interaction.edge_interaction_width, zoom)
            .max(self.style.wire_width / zoom);
        let threshold2 = hit_w * hit_w;

        index.query_edges(pos, hit_w, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(EdgeId, f32)> = None;
        for &edge_id in scratch.iter() {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };

            let route = self.edge_render_hint(graph, edge_id).route;
            let d2 = match route {
                EdgeRouteKind::Bezier => wire_distance2(pos, from, to, zoom, bezier_steps),
                EdgeRouteKind::Straight => dist2_point_to_segment(pos, from, to),
                EdgeRouteKind::Step => step_wire_distance2(pos, from, to),
            };
            if d2 <= threshold2 {
                match best {
                    Some((_id, best_d2)) if best_d2 <= d2 => {}
                    _ => best = Some((edge_id, d2)),
                }
            }
        }

        best.map(|(id, _)| id)
    }

    pub(super) fn hit_edge_focus_anchor(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialIndex,
        pos: Point,
        zoom: f32,
        scratch: &mut Vec<EdgeId>,
    ) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
        let z = zoom.max(1.0e-6);
        let half =
            (0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN + Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN) / z;
        let query_r = (half * 1.5).max(half);
        index.query_edges(pos, query_r, scratch);
        scratch.sort_unstable();
        scratch.dedup();

        let mut best: Option<(EdgeId, EdgeEndpoint, PortId, f32)> = None;

        for &edge_id in scratch.iter() {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let (allow_source, allow_target) =
                Self::edge_reconnectable_flags(edge, &snapshot.interaction);
            if !allow_source && !allow_target {
                continue;
            }
            let Some(from) = geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = geom.port_center(edge.to) else {
                continue;
            };

            let route = self.edge_render_hint(graph, edge_id).route;
            let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
            let r0 = Self::edge_focus_anchor_rect(a0, zoom);
            let r1 = Self::edge_focus_anchor_rect(a1, zoom);

            let mut consider =
                |center: Point, rect: Rect, endpoint: EdgeEndpoint, fixed: PortId| {
                    if !rect.contains(pos) {
                        return;
                    }
                    let dx = pos.x.0 - center.x.0;
                    let dy = pos.y.0 - center.y.0;
                    let d2 = dx * dx + dy * dy;
                    match best {
                        Some((_id, _ep, _fixed, best_d2)) if best_d2 <= d2 => {}
                        _ => best = Some((edge_id, endpoint, fixed, d2)),
                    }
                };

            if allow_source {
                consider(a0, r0, EdgeEndpoint::From, edge.to);
            }
            if allow_target {
                consider(a1, r1, EdgeEndpoint::To, edge.from);
            }
        }

        best.map(|(id, endpoint, fixed, _)| (id, endpoint, fixed))
    }

    pub(super) fn pick_reconnect_endpoint(
        &self,
        graph: &Graph,
        geom: &CanvasGeometry,
        edge_id: EdgeId,
        pos: Point,
        reconnect_radius_screen: f32,
        zoom: f32,
    ) -> Option<(EdgeEndpoint, PortId)> {
        let edge = graph.edges.get(&edge_id)?;

        let from_center = geom.port_center(edge.from);
        let to_center = geom.port_center(edge.to);

        let (from_center, to_center) = match (from_center, to_center) {
            (Some(a), Some(b)) => (a, b),
            _ => return None,
        };

        let d2_from = {
            let dx = pos.x.0 - from_center.x.0;
            let dy = pos.y.0 - from_center.y.0;
            dx * dx + dy * dy
        };
        let d2_to = {
            let dx = pos.x.0 - to_center.x.0;
            let dy = pos.y.0 - to_center.y.0;
            dx * dx + dy * dy
        };

        if reconnect_radius_screen.is_finite() && reconnect_radius_screen > 0.0 {
            let r = canvas_units_from_screen_px(reconnect_radius_screen, zoom);
            let r2 = r * r;
            let min_d2 = d2_from.min(d2_to);
            if min_d2 > r2 {
                return None;
            }
        }

        if d2_from <= d2_to {
            Some((EdgeEndpoint::From, edge.to))
        } else {
            Some((EdgeEndpoint::To, edge.from))
        }
    }
}
