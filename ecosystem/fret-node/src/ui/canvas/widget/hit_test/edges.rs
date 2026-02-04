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
        let hit_w = canvas_units_from_screen_px(snapshot.interaction.edge_interaction_width, zoom)
            .max(self.style.wire_width / zoom);
        let threshold2 = hit_w * hit_w;

        let candidates = ctx
            .index
            .query_edges_sorted_dedup(pos, hit_w, ctx.scratch.edges_mut());
        let mut best = score::BestEdgeByDistance::new(zoom);

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
            let d2 = if let Some(custom) =
                self.edge_custom_path(graph, edge_id, &hint, from, to, zoom)
            {
                wire_distance2_path(pos, &custom.commands, bezier_steps)
            } else {
                match hint.route {
                    EdgeRouteKind::Bezier => wire_distance2(pos, from, to, zoom, bezier_steps),
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
