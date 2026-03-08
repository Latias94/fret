use super::*;

pub(super) fn required_port_direction_from_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<PortDirection> {
    canvas.interaction.wire_drag.as_ref().and_then(|wire_drag| {
        let source_port = match &wire_drag.kind {
            WireDragKind::New { from, .. } => Some(*from),
            WireDragKind::Reconnect { fixed, .. } => Some(*fixed),
            WireDragKind::ReconnectMany { edges } => edges.first().map(|edge| edge.2),
        }?;
        let source_dir = canvas
            .graph
            .read_ref(host, |graph| {
                graph.ports.get(&source_port).map(|port| port.dir)
            })
            .ok()
            .flatten()?;
        Some(opposite_port_direction(source_dir))
    })
}

pub(super) fn directional_port_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    from_port: PortId,
    from_center: CanvasPoint,
    dir: PortNavDir,
    required_dir: Option<PortDirection>,
) -> Option<PortId> {
    let (geom, _) = canvas.canvas_derived(host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut best: Option<DirectionalPortRank> = None;

            for (&port, handle) in &geom.ports {
                if port == from_port {
                    continue;
                }
                let Some(graph_port) = graph.ports.get(&port) else {
                    continue;
                };
                if let Some(required_dir) = required_dir
                    && graph_port.dir != required_dir
                {
                    continue;
                }

                let Some(candidate) = rank_directional_port_candidate(
                    port,
                    from_center,
                    CanvasPoint {
                        x: handle.center.x.0,
                        y: handle.center.y.0,
                    },
                    dir,
                ) else {
                    continue;
                };

                if is_better_directional_port_rank(candidate, best) {
                    best = Some(candidate);
                }
            }

            best.map(|rank| rank.port)
        })
        .ok()
        .flatten()
}

#[derive(Clone, Copy)]
struct DirectionalPortRank {
    angle: f32,
    parallel: f32,
    dist2: f32,
    port: PortId,
}

fn opposite_port_direction(direction: PortDirection) -> PortDirection {
    match direction {
        PortDirection::In => PortDirection::Out,
        PortDirection::Out => PortDirection::In,
    }
}

fn rank_directional_port_candidate(
    port: PortId,
    from_center: CanvasPoint,
    candidate_center: CanvasPoint,
    dir: PortNavDir,
) -> Option<DirectionalPortRank> {
    let dx = candidate_center.x - from_center.x;
    let dy = candidate_center.y - from_center.y;
    let (parallel, perp) = match dir {
        PortNavDir::Left => (-dx, dy.abs()),
        PortNavDir::Right => (dx, dy.abs()),
        PortNavDir::Up => (-dy, dx.abs()),
        PortNavDir::Down => (dy, dx.abs()),
    };
    if !parallel.is_finite() || !perp.is_finite() || parallel <= 1.0e-6 {
        return None;
    }

    let angle = (perp / parallel).abs();
    let dist2 = dx * dx + dy * dy;
    if !angle.is_finite() || !dist2.is_finite() {
        return None;
    }

    Some(DirectionalPortRank {
        angle,
        parallel,
        dist2,
        port,
    })
}

fn is_better_directional_port_rank(
    candidate: DirectionalPortRank,
    best: Option<DirectionalPortRank>,
) -> bool {
    let Some(best) = best else {
        return true;
    };
    let by_angle = candidate.angle.total_cmp(&best.angle);
    if by_angle != std::cmp::Ordering::Equal {
        return by_angle == std::cmp::Ordering::Less;
    }
    let by_parallel = candidate.parallel.total_cmp(&best.parallel);
    if by_parallel != std::cmp::Ordering::Equal {
        return by_parallel == std::cmp::Ordering::Less;
    }
    let by_dist = candidate.dist2.total_cmp(&best.dist2);
    if by_dist != std::cmp::Ordering::Equal {
        return by_dist == std::cmp::Ordering::Less;
    }
    candidate.port < best.port
}
