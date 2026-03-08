use super::*;

pub(super) fn required_port_direction_from_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<PortDirection> {
    super::focus_port_direction_wire::required_port_direction_from_wire_drag(canvas, host)
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
            let mut best = None;

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

                let Some(candidate) =
                    super::focus_port_direction_rank::rank_directional_port_candidate(
                        port,
                        from_center,
                        CanvasPoint {
                            x: handle.center.x.0,
                            y: handle.center.y.0,
                        },
                        dir,
                    )
                else {
                    continue;
                };

                if super::focus_port_direction_rank::is_better_directional_port_rank(
                    candidate, best,
                ) {
                    best = Some(candidate);
                }
            }

            best.map(|rank| rank.port)
        })
        .ok()
        .flatten()
}
