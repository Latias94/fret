use super::*;

pub(super) fn refresh_focused_port_hints<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    canvas.interaction.focused_port_valid = false;
    canvas.interaction.focused_port_convertible = false;

    let snapshot = canvas.sync_view_state(host);
    let mode = snapshot.interaction.connection_mode;

    let Some(target) = canvas.interaction.focused_port else {
        return;
    };
    let Some(wire_drag) = canvas.interaction.wire_drag.clone() else {
        return;
    };

    let presenter = &mut *canvas.presenter;
    let (valid, convertible) = canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = graph.clone();

            let valid = match &wire_drag.kind {
                WireDragKind::New { from, bundle } => {
                    let sources = if bundle.is_empty() {
                        std::slice::from_ref(from)
                    } else {
                        bundle.as_slice()
                    };
                    let mut any_accept = false;
                    for src in sources {
                        let plan = presenter.plan_connect(&scratch, *src, target, mode);
                        if plan.decision != ConnectDecision::Accept {
                            continue;
                        }
                        any_accept = true;
                        let tx = GraphTransaction {
                            label: None,
                            ops: plan.ops.clone(),
                        };
                        let _ = apply_transaction(&mut scratch, &tx);
                    }
                    any_accept
                }
                WireDragKind::Reconnect { edge, endpoint, .. } => matches!(
                    presenter
                        .plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode)
                        .decision,
                    ConnectDecision::Accept
                ),
                WireDragKind::ReconnectMany { edges } => {
                    let mut any_accept = false;
                    for (edge, endpoint, _fixed) in edges {
                        let plan =
                            presenter.plan_reconnect_edge(&scratch, *edge, *endpoint, target, mode);
                        if plan.decision != ConnectDecision::Accept {
                            continue;
                        }
                        any_accept = true;
                        let tx = GraphTransaction {
                            label: None,
                            ops: plan.ops.clone(),
                        };
                        let _ = apply_transaction(&mut scratch, &tx);
                    }
                    any_accept
                }
            };

            let convertible = if !valid {
                match &wire_drag.kind {
                    WireDragKind::New { from, bundle } if bundle.len() <= 1 => {
                        conversion::is_convertible(presenter, &scratch, *from, target)
                    }
                    _ => false,
                }
            } else {
                false
            };

            (valid, convertible)
        })
        .ok()
        .unwrap_or((false, false));

    if canvas.interaction.wire_drag.is_some() && canvas.interaction.focused_port == Some(target) {
        canvas.interaction.focused_port_valid = valid;
        canvas.interaction.focused_port_convertible = convertible;
    }
}

pub(super) fn port_center_canvas<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Option<CanvasPoint> {
    let (geom, _) = canvas.canvas_derived(&*host, snapshot);
    geom.ports.get(&port).map(|h| CanvasPoint {
        x: h.center.x.0,
        y: h.center.y.0,
    })
}

pub(super) fn activate_focused_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    if !snapshot.interaction.elements_selectable {
        return false;
    }

    let Some(port) = canvas
        .interaction
        .focused_port
        .or(canvas.interaction.hover_port)
    else {
        return false;
    };

    let pos = port_center_canvas(canvas, cx.app, snapshot, port)
        .map(|p| Point::new(Px(p.x), Px(p.y)))
        .or(canvas.interaction.last_pos)
        .unwrap_or_else(|| {
            let bounds = canvas.interaction.last_bounds.unwrap_or_default();
            Point::new(
                Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
                Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
            )
        });

    if canvas.interaction.wire_drag.is_none() {
        canvas.interaction.wire_drag = Some(WireDrag {
            kind: WireDragKind::New {
                from: port,
                bundle: Vec::new(),
            },
            pos,
        });
        canvas.interaction.click_connect = true;
        canvas.interaction.pending_wire_drag = None;
        canvas.interaction.suspended_wire_drag = None;
        canvas.interaction.sticky_wire = false;
        canvas.interaction.sticky_wire_ignore_next_up = false;
        canvas.interaction.focused_edge = None;
        canvas.interaction.focused_port = None;
        canvas.interaction.focused_port_valid = false;
        canvas.interaction.focused_port_convertible = false;
        canvas.interaction.hover_port = None;
        canvas.interaction.hover_port_valid = false;
        canvas.interaction.hover_port_convertible = false;
        canvas.interaction.hover_port_diagnostic = None;
        return true;
    }

    if let Some(mut w) = canvas.interaction.wire_drag.take() {
        w.pos = pos;
        canvas.interaction.wire_drag = Some(w);
    }

    let _ = wire_drag::handle_wire_left_up_with_forced_target(
        canvas,
        cx,
        snapshot,
        snapshot.zoom,
        Some(port),
    );
    refresh_focused_port_hints(canvas, cx.app);
    true
}
