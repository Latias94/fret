use super::prelude::*;

pub(super) fn commit_reconnect_many<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    edges: Vec<(EdgeId, EdgeEndpoint, PortId)>,
    target: Option<PortId>,
) -> CommitEmit {
    let window = cx.window();
    let mut connect_end_outcome = ConnectEndOutcome::NoOp;
    let mut connect_end_target = target;

    if let Some(target) = target {
        connect_end_target = Some(target);
        let presenter = &mut *canvas.presenter;
        let (ops_all, toast) = canvas
            .graph
            .read_ref(cx.host(), |graph| {
                let mut scratch = graph.clone();
                let mut ops_all: Vec<GraphOp> = Vec::new();
                let mut toast: Option<(DiagnosticSeverity, Arc<str>)> = None;

                for (edge, endpoint, _fixed) in edges {
                    let plan = presenter.plan_reconnect_edge(
                        &scratch,
                        edge,
                        endpoint,
                        target,
                        snapshot.interaction.connection_mode,
                    );
                    match plan.decision {
                        ConnectDecision::Accept => {
                            let tx = GraphTransaction {
                                label: None,
                                ops: plan.ops.clone(),
                            };
                            let _ = apply_transaction(&mut scratch, &tx);
                            ops_all.extend(plan.ops);
                        }
                        ConnectDecision::Reject => {
                            if toast.is_none() {
                                toast = NodeGraphCanvasWith::<M>::toast_from_diagnostics(
                                    &plan.diagnostics,
                                );
                            }
                        }
                    }
                }

                (ops_all, toast)
            })
            .ok()
            .unwrap_or_default();

        if !ops_all.is_empty() {
            canvas.apply_ops(cx.host(), window, ops_all);
            connect_end_outcome = ConnectEndOutcome::Committed;
        }
        if let Some((sev, msg)) = toast {
            canvas.show_toast(cx.host(), window, sev, msg);
        }
    } else if snapshot.interaction.reconnect_on_drop_empty {
        let ops_all = canvas
            .graph
            .read_ref(cx.host(), |graph| {
                let mut out: Vec<GraphOp> = Vec::new();
                out.reserve(edges.len());
                for (edge_id, _endpoint, _fixed) in edges {
                    let Some(edge_value) = graph.edges.get(&edge_id) else {
                        continue;
                    };
                    out.push(GraphOp::RemoveEdge {
                        id: edge_id,
                        edge: edge_value.clone(),
                    });
                }
                out
            })
            .ok()
            .unwrap_or_default();

        if !ops_all.is_empty() {
            let label = if ops_all.len() == 1 {
                "Disconnect Edge"
            } else {
                "Disconnect Edges"
            };
            let _ = canvas.commit_ops(cx.host(), window, Some(label), ops_all);
            connect_end_outcome = ConnectEndOutcome::Committed;
        }
    }

    CommitEmit {
        target: connect_end_target,
        outcome: connect_end_outcome,
    }
}
