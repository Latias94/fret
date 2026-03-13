use super::super::prelude::*;

pub(super) fn compute_hover_validity_and_diag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
    hover_port: Option<PortId>,
) -> (bool, Option<(DiagnosticSeverity, Arc<str>)>) {
    let Some(target) = hover_port else {
        return (false, None);
    };

    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            if !NodeGraphCanvasWith::<M>::port_is_connectable_end(
                graph,
                &snapshot.interaction,
                target,
            ) {
                return (
                    false,
                    Some((
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("Port is not connectable"),
                    )),
                );
            }

            let mut best_diag: Option<(DiagnosticSeverity, Arc<str>)> = None;
            let mut accept = false;

            let mut consider_diag = |plan: &crate::rules::ConnectPlan| {
                for d in plan.diagnostics.iter() {
                    let next = (d.severity, Arc::<str>::from(d.message.clone()));
                    match &best_diag {
                        Some((best_sev, _)) if severity_rank(*best_sev) > severity_rank(next.0) => {
                        }
                        _ => best_diag = Some(next),
                    }
                }
            };

            match kind {
                WireDragKind::New { from, bundle } => {
                    let sources = if bundle.is_empty() {
                        std::slice::from_ref(from)
                    } else {
                        bundle.as_slice()
                    };
                    for src in sources {
                        let plan = presenter.can_connect(
                            graph,
                            *src,
                            target,
                            snapshot.interaction.connection_mode,
                        );
                        if plan.decision == ConnectDecision::Accept {
                            accept = true;
                            break;
                        }
                        consider_diag(&plan);
                    }
                }
                WireDragKind::Reconnect { edge, endpoint, .. } => {
                    let plan = presenter.can_reconnect_edge(
                        graph,
                        *edge,
                        *endpoint,
                        target,
                        snapshot.interaction.connection_mode,
                    );
                    if plan.decision == ConnectDecision::Accept {
                        accept = true;
                    } else {
                        consider_diag(&plan);
                    }
                }
                WireDragKind::ReconnectMany { edges } => {
                    for (edge, endpoint, _fixed) in edges {
                        let plan = presenter.can_reconnect_edge(
                            graph,
                            *edge,
                            *endpoint,
                            target,
                            snapshot.interaction.connection_mode,
                        );
                        if plan.decision == ConnectDecision::Accept {
                            accept = true;
                            break;
                        }
                        consider_diag(&plan);
                    }
                }
            }

            if accept {
                (true, None)
            } else {
                let diag = best_diag.or_else(|| {
                    Some((
                        DiagnosticSeverity::Error,
                        Arc::<str>::from("Invalid connection"),
                    ))
                });
                (false, diag)
            }
        })
        .ok()
        .unwrap_or((false, None))
}
