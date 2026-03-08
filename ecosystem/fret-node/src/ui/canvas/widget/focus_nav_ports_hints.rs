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
                    for source in sources {
                        let plan = presenter.plan_connect(&scratch, *source, target, mode);
                        if plan.decision != ConnectDecision::Accept {
                            continue;
                        }
                        any_accept = true;
                        apply_plan_ops(&mut scratch, plan.ops.clone());
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
                        apply_plan_ops(&mut scratch, plan.ops.clone());
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

fn apply_plan_ops(graph: &mut Graph, ops: Vec<GraphOp>) {
    let tx = GraphTransaction { label: None, ops };
    let _ = apply_transaction(graph, &tx);
}
