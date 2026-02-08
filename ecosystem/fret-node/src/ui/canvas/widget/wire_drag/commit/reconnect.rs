use super::prelude::*;

pub(super) fn commit_reconnect<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WireCommitCx<H>,
    snapshot: &ViewSnapshot,
    edge: EdgeId,
    endpoint: EdgeEndpoint,
    target: Option<PortId>,
) -> CommitEmit {
    let window = cx.window();
    let mut connect_end_outcome = ConnectEndOutcome::NoOp;
    let mut connect_end_target = target;

    if let Some(target) = target {
        connect_end_target = Some(target);
        enum Outcome {
            Apply(Vec<GraphOp>),
            Reject(DiagnosticSeverity, Arc<str>),
            Ignore,
        }

        let outcome = {
            let presenter = &mut *canvas.presenter;
            canvas
                .graph
                .read_ref(cx.host(), |graph| {
                    let plan = presenter.plan_reconnect_edge(
                        graph,
                        edge,
                        endpoint,
                        target,
                        snapshot.interaction.connection_mode,
                    );
                    match plan.decision {
                        ConnectDecision::Accept => Outcome::Apply(plan.ops),
                        ConnectDecision::Reject => {
                            NodeGraphCanvasWith::<M>::toast_from_diagnostics(&plan.diagnostics)
                                .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                .unwrap_or(Outcome::Ignore)
                        }
                    }
                })
                .ok()
                .unwrap_or(Outcome::Ignore)
        };
        match outcome {
            Outcome::Apply(ops) => {
                canvas.apply_ops(cx.host(), window, ops);
                connect_end_outcome = ConnectEndOutcome::Committed;
            }
            Outcome::Reject(sev, msg) => {
                connect_end_outcome = ConnectEndOutcome::Rejected;
                canvas.show_toast(cx.host(), window, sev, msg);
            }
            Outcome::Ignore => {}
        }
    } else if snapshot.interaction.reconnect_on_drop_empty {
        let ops = canvas
            .graph
            .read_ref(cx.host(), |graph| {
                let Some(edge_value) = graph.edges.get(&edge) else {
                    return Vec::new();
                };
                vec![GraphOp::RemoveEdge {
                    id: edge,
                    edge: edge_value.clone(),
                }]
            })
            .ok()
            .unwrap_or_default();
        if !ops.is_empty() {
            let _ = canvas.commit_ops(cx.host(), window, Some("Disconnect Edge"), ops);
            connect_end_outcome = ConnectEndOutcome::Committed;
        }
    }

    CommitEmit {
        target: connect_end_target,
        outcome: connect_end_outcome,
    }
}
