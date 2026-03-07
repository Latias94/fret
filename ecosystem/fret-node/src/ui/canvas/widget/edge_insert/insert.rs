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
        let at = insert_candidate_canvas_point(canvas, &candidate, invoked_at);

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
            let node_id = is_reroute_insert_candidate(&candidate)
                .then(|| NodeGraphCanvasWith::<M>::first_added_node_id(&ops))
                .flatten();
            canvas.apply_ops(cx.app, cx.window, ops);
            canvas.select_inserted_node(cx.app, node_id);
        }
        Outcome::Reject(sev, msg) => canvas.show_toast(cx.app, cx.window, sev, msg),
        Outcome::Ignore => {}
    }
}
