use super::prelude::*;

pub(in super::super) fn insert_node_on_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
    candidate: InsertNodeCandidate,
) {
    enum Outcome {
        Apply(Vec<GraphOp>),
        Reject(DiagnosticSeverity, Arc<str>),
        Ignore,
    }

    canvas.record_recent_kind(&candidate.kind);

    let outcome = {
        let at = if candidate.kind.0 == REROUTE_KIND {
            canvas.reroute_pos_for_invoked_at(invoked_at)
        } else {
            CanvasPoint {
                x: invoked_at.x.0,
                y: invoked_at.y.0,
            }
        };

        let presenter = &mut *canvas.presenter;
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                let plan = presenter.plan_split_edge_candidate(graph, edge, &candidate, at);
                match plan.decision {
                    ConnectDecision::Accept => Outcome::Apply(plan.ops),
                    ConnectDecision::Reject => {
                        NodeGraphCanvasWith::<M>::toast_from_diagnostics(&plan.diagnostics)
                            .map(|(sev, msg)| Outcome::Reject(sev, msg))
                            .unwrap_or_else(|| {
                                Outcome::Reject(
                                    DiagnosticSeverity::Error,
                                    Arc::<str>::from(format!(
                                        "node insertion was rejected: {}",
                                        candidate.kind.0
                                    )),
                                )
                            })
                    }
                }
            })
            .ok()
            .unwrap_or(Outcome::Ignore)
    };

    match outcome {
        Outcome::Apply(ops) => {
            let select_node = candidate.kind.0 == REROUTE_KIND;
            let node_id = select_node
                .then(|| NodeGraphCanvasWith::<M>::first_added_node_id(&ops))
                .flatten();
            canvas.apply_ops(cx.app, cx.window, ops);
            if let Some(node_id) = node_id {
                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_groups.clear();
                    s.selected_nodes.clear();
                    s.selected_nodes.push(node_id);
                    s.draw_order.retain(|id| *id != node_id);
                    s.draw_order.push(node_id);
                });
            }
        }
        Outcome::Reject(sev, msg) => canvas.show_toast(cx.app, cx.window, sev, msg),
        Outcome::Ignore => {}
    }
}
