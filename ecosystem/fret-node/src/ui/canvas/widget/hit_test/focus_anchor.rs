use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn hit_edge_focus_anchor(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        ctx: &mut HitTestCtx<'_>,
        pos: Point,
    ) -> Option<(EdgeId, EdgeEndpoint, PortId)> {
        let zoom = ctx.zoom;
        let z = zoom.max(1.0e-6);
        let half =
            (0.5 * Self::EDGE_FOCUS_ANCHOR_SIZE_SCREEN + Self::EDGE_FOCUS_ANCHOR_PAD_SCREEN) / z;
        let query_r = (half * 1.5).max(half);
        let candidates = ctx
            .index
            .query_edges_sorted_dedup(pos, query_r, ctx.scratch.edges_mut());
        let mut best = score::BestEdgeFocusAnchorByDistance::new(zoom);

        for &edge_id in candidates {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let (allow_source, allow_target) =
                Self::edge_reconnectable_flags(edge, &snapshot.interaction);
            if !allow_source && !allow_target {
                continue;
            }
            let Some(from) = ctx.geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = ctx.geom.port_center(edge.to) else {
                continue;
            };

            let hint = self.edge_render_hint(graph, edge_id);
            let (a0, a1) = if let Some(custom) =
                self.edge_custom_path(graph, edge_id, &hint, from, to, zoom)
            {
                if let Some((t0, t1)) = path_start_end_tangents(&custom.commands) {
                    Self::edge_focus_anchor_centers_from_tangents(from, to, zoom, t0, t1)
                } else {
                    Self::edge_focus_anchor_centers(hint.route, from, to, zoom)
                }
            } else {
                Self::edge_focus_anchor_centers(hint.route, from, to, zoom)
            };
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
                    best.consider(edge_id, endpoint, fixed, d2);
                };

            if allow_source {
                consider(a0, r0, EdgeEndpoint::From, edge.to);
            }
            if allow_target {
                consider(a1, r1, EdgeEndpoint::To, edge.from);
            }
        }

        best.into_value()
    }

    pub(in super::super) fn pick_reconnect_endpoint(
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

        let mut best = score::BestEdgeEndpointByDistance::new(zoom);
        best.consider(EdgeEndpoint::From, edge.to, d2_from);
        best.consider(EdgeEndpoint::To, edge.from, d2_to);
        best.into_value()
    }
}
