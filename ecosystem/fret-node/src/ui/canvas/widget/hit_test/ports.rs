use super::score::{BestLoosePort, BestPortByNodeRank};
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn hit_port(
        &self,
        ctx: &mut HitTestCtx<'_>,
        pos: Point,
    ) -> Option<PortId> {
        let zoom = ctx.zoom;
        let r = self.style.pin_radius / zoom;
        if !r.is_finite() || r <= 0.0 {
            return None;
        }

        let candidates = ctx
            .index
            .query_ports_sorted_dedup(pos, r, ctx.scratch.ports_mut());

        let mut best = BestPortByNodeRank::new();
        for &port_id in candidates {
            let Some(handle) = ctx.geom.ports.get(&port_id) else {
                continue;
            };
            if !Self::rect_contains_point(handle.bounds, pos) {
                continue;
            }
            let rank = ctx.geom.node_rank.get(&handle.node).copied().unwrap_or(0);
            best.consider(port_id, rank);
        }

        best.into_port_id()
    }

    fn pick_loose_port_in_radius(
        &self,
        graph: &Graph,
        interaction: &crate::io::NodeGraphInteractionState,
        ctx: &mut HitTestCtx<'_>,
        from: PortId,
        desired_dir: PortDirection,
        pos: Point,
        require_connectable_end: bool,
    ) -> Option<PortId> {
        let zoom = ctx.zoom;
        let radius_screen = interaction.connection_radius;
        if !radius_screen.is_finite() || radius_screen <= 0.0 {
            return None;
        }

        let r = canvas_units_from_screen_px(radius_screen, zoom);
        let r2 = r * r;

        let mut best = BestLoosePort::new(zoom);
        let candidates = ctx
            .index
            .query_ports_sorted_dedup(pos, r, ctx.scratch.ports_mut());
        for &port_id in candidates {
            if port_id == from {
                continue;
            }
            let Some(handle) = ctx.geom.ports.get(&port_id) else {
                continue;
            };
            if require_connectable_end
                && !Self::port_is_connectable_end(graph, interaction, port_id)
            {
                continue;
            }
            let d2 = Self::distance_sq_point_to_rect(pos, handle.bounds);
            if d2 > r2 {
                continue;
            }
            let preferred = handle.dir == desired_dir;
            let rank = ctx.geom.node_rank.get(&handle.node).copied().unwrap_or(0);
            best.consider(port_id, d2, preferred, rank);
        }

        best.into_port_id()
    }

    pub(in super::super) fn pick_target_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        ctx: &mut HitTestCtx<'_>,
        from: PortId,
        require_from_connectable_start: bool,
        pos: Point,
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
                let candidate = self.hit_port(ctx, pos)?;
                let port = graph.ports.get(&candidate)?;
                (candidate != from
                    && port.dir == desired_dir
                    && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                .then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(ctx, pos)?;
                    return (candidate != from
                        && Self::port_is_connectable_end(graph, &snapshot.interaction, candidate))
                    .then_some(candidate);
                }

                self.pick_loose_port_in_radius(
                    graph,
                    &snapshot.interaction,
                    ctx,
                    from,
                    desired_dir,
                    pos,
                    true,
                )
            }
        }
    }

    pub(in super::super) fn pick_wire_hover_port(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        ctx: &mut HitTestCtx<'_>,
        from: PortId,
        require_from_connectable_start: bool,
        pos: Point,
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
                let candidate = self.hit_port(ctx, pos)?;
                (candidate != from).then_some(candidate)
            }
            NodeGraphConnectionMode::Loose => {
                let radius_screen = snapshot.interaction.connection_radius;
                if !radius_screen.is_finite() || radius_screen <= 0.0 {
                    let candidate = self.hit_port(ctx, pos)?;
                    return (candidate != from).then_some(candidate);
                }

                self.pick_loose_port_in_radius(
                    graph,
                    &snapshot.interaction,
                    ctx,
                    from,
                    desired_dir,
                    pos,
                    false,
                )
            }
        }
    }
}
