use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn hit_edge(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        ctx: &mut HitTestCtx<'_>,
        pos: Point,
    ) -> Option<EdgeId> {
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let zoom = ctx.zoom;
        let z = zoom_z(zoom);
        let max_override = self
            .geometry_overrides
            .as_ref()
            .map(|o| o.max_edge_interaction_width_override_px())
            .unwrap_or(0.0);
        let hit_w_query = hit_test_canvas_units_from_screen_px(
            snapshot
                .interaction
                .edge_interaction_width
                .max(max_override),
            z,
        )
        .max(self.style.geometry.wire_width / z);

        let candidates =
            ctx.index
                .query_edges_sorted_dedup(pos, hit_w_query, ctx.scratch.edges_mut());
        let mut best = score::BestEdgeByDistance::new(z);

        for &edge_id in candidates {
            let Some(edge) = graph.edges.get(&edge_id) else {
                continue;
            };
            let Some(from) = ctx.geom.port_center(edge.from) else {
                continue;
            };
            let Some(to) = ctx.geom.port_center(edge.to) else {
                continue;
            };

            let hint = self.edge_render_hint(graph, edge_id);
            let interaction_width_px = self
                .geometry_overrides
                .as_ref()
                .and_then(|o| o.edge_geometry_override(edge_id).interaction_width_px)
                .unwrap_or(snapshot.interaction.edge_interaction_width);
            let hit_w_edge = hit_test_canvas_units_from_screen_px(interaction_width_px, z)
                .max(self.style.geometry.wire_width / z);
            let threshold2 = hit_w_edge * hit_w_edge;
            let d2 = if let Some(custom) = self.edge_custom_path(graph, edge_id, &hint, from, to, z)
            {
                wire_distance2_path(pos, &custom.commands, bezier_steps)
            } else {
                match hint.route {
                    EdgeRouteKind::Bezier => wire_distance2(pos, from, to, z, bezier_steps),
                    EdgeRouteKind::Straight => dist2_point_to_segment(pos, from, to),
                    EdgeRouteKind::Step => step_wire_distance2(pos, from, to),
                }
            };
            if d2 <= threshold2 {
                best.consider(edge_id, d2);
            }
        }

        best.into_edge_id()
    }
}
