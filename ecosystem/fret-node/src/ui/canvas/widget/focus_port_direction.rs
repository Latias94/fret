use super::*;

pub(super) fn focus_port_direction<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    dir: PortNavDir,
) -> bool {
    if !snapshot.interaction.elements_selectable {
        return false;
    }

    if canvas.interaction.focused_port.is_none() {
        return canvas.focus_next_port(host, true);
    }

    let Some(from_port) = canvas.interaction.focused_port else {
        return false;
    };

    let Some(from_center) = canvas.port_center_canvas(host, snapshot, from_port) else {
        return false;
    };

    let required_dir = required_port_direction_from_wire_drag(canvas, host);
    let Some(next) = directional_port_candidate(
        canvas,
        host,
        snapshot,
        from_port,
        from_center,
        dir,
        required_dir,
    ) else {
        return false;
    };

    let owner = canvas
        .graph
        .read_ref(host, |g| g.ports.get(&next).map(|p| p.node))
        .ok()
        .flatten();

    let Some(owner) = owner else {
        return false;
    };

    canvas.interaction.focused_node = Some(owner);
    canvas.interaction.focused_edge = None;
    canvas.interaction.focused_port = Some(next);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |s| {
        s.selected_edges.clear();
        s.selected_groups.clear();
        s.selected_nodes.clear();
        s.selected_nodes.push(owner);
        s.draw_order.retain(|id| *id != owner);
        s.draw_order.push(owner);
    });

    let snapshot = canvas.sync_view_state(host);
    if let Some(center) = canvas.port_center_canvas(host, &snapshot, next) {
        canvas.ensure_canvas_point_visible(host, &snapshot, center);
    }

    true
}

fn required_port_direction_from_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
            .read_ref(host, |g| g.ports.get(&source_port).map(|p| p.dir))
            .ok()
            .flatten()?;
        Some(match source_dir {
            PortDirection::In => PortDirection::Out,
            PortDirection::Out => PortDirection::In,
        })
    })
}

fn directional_port_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    from_port: PortId,
    from_center: CanvasPoint,
    dir: PortNavDir,
    required_dir: Option<PortDirection>,
) -> Option<PortId> {
    #[derive(Clone, Copy)]
    struct Rank {
        angle: f32,
        parallel: f32,
        dist2: f32,
        port: PortId,
    }

    let (geom, _) = canvas.canvas_derived(host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut best: Option<Rank> = None;

            for (&port, handle) in &geom.ports {
                if port == from_port {
                    continue;
                }

                let Some(graph_port) = graph.ports.get(&port) else {
                    continue;
                };

                if let Some(required_dir) = required_dir {
                    if graph_port.dir != required_dir {
                        continue;
                    }
                }

                let dx = handle.center.x.0 - from_center.x;
                let dy = handle.center.y.0 - from_center.y;
                let (parallel, perp) = match dir {
                    PortNavDir::Left => (-dx, dy.abs()),
                    PortNavDir::Right => (dx, dy.abs()),
                    PortNavDir::Up => (-dy, dx.abs()),
                    PortNavDir::Down => (dy, dx.abs()),
                };
                if !parallel.is_finite() || !perp.is_finite() || parallel <= 1.0e-6 {
                    continue;
                }

                let angle = (perp / parallel).abs();
                let dist2 = dx * dx + dy * dy;
                if !angle.is_finite() || !dist2.is_finite() {
                    continue;
                }

                let candidate = Rank {
                    angle,
                    parallel,
                    dist2,
                    port,
                };

                let better = match best {
                    None => true,
                    Some(best) => {
                        let by_angle = angle.total_cmp(&best.angle);
                        if by_angle != std::cmp::Ordering::Equal {
                            by_angle == std::cmp::Ordering::Less
                        } else {
                            let by_parallel = parallel.total_cmp(&best.parallel);
                            if by_parallel != std::cmp::Ordering::Equal {
                                by_parallel == std::cmp::Ordering::Less
                            } else {
                                let by_dist = dist2.total_cmp(&best.dist2);
                                if by_dist != std::cmp::Ordering::Equal {
                                    by_dist == std::cmp::Ordering::Less
                                } else {
                                    port < best.port
                                }
                            }
                        }
                    }
                };

                if better {
                    best = Some(candidate);
                }
            }

            best.map(|rank| rank.port)
        })
        .ok()
        .flatten()
}
