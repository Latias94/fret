use super::prelude::*;

pub(super) fn from_port_and_require_from_connectable_start(
    kind: &WireDragKind,
) -> (Option<PortId>, bool) {
    match kind {
        WireDragKind::New { from, .. } => (Some(*from), true),
        WireDragKind::Reconnect { fixed, .. } => (Some(*fixed), false),
        WireDragKind::ReconnectMany { edges } => (edges.first().map(|e| e.2), false),
    }
}

pub(super) fn pick_hover_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    index: &CanvasSpatialIndex,
    zoom: f32,
    from_port: Option<PortId>,
    require_from_connectable_start: bool,
    pos: Point,
) -> Option<PortId> {
    let Some(from_port) = from_port else {
        return None;
    };

    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom, index, zoom, &mut scratch);
            canvas.pick_wire_hover_port(
                graph,
                snapshot,
                &mut ctx,
                from_port,
                require_from_connectable_start,
                pos,
            )
        })
        .ok()
        .flatten()
}

pub(super) fn pick_hover_edge_if_no_hover_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    geom: &CanvasGeometry,
    index: &CanvasSpatialIndex,
    zoom: f32,
    pos: Point,
    hover_port: Option<PortId>,
) -> Option<EdgeId> {
    if hover_port.is_some() {
        return None;
    }

    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom, index, zoom, &mut scratch);
            canvas.hit_edge(graph, snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten()
}

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

pub(super) fn compute_hover_convertible<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    kind: &WireDragKind,
    hover_port: Option<PortId>,
    hover_valid: bool,
) -> bool {
    if hover_valid {
        return false;
    }

    let Some(target) = hover_port else {
        return false;
    };

    let WireDragKind::New { from, bundle } = kind else {
        return false;
    };
    if bundle.len() > 1 {
        return false;
    }

    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            if !NodeGraphCanvasWith::<M>::port_is_connectable_end(
                graph,
                &snapshot.interaction,
                target,
            ) {
                return false;
            }
            conversion::is_convertible(presenter, graph, *from, target)
        })
        .ok()
        .unwrap_or(false)
}
