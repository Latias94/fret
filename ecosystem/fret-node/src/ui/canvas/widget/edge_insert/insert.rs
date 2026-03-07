use super::prelude::*;

pub(in super::super) fn activate_edge_insert_picker_action<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge: EdgeId,
    invoked_at: Point,
    action: NodeGraphContextMenuAction,
    menu_candidates: &[InsertNodeCandidate],
) -> bool {
    match action {
        NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
            let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                return true;
            };
            insert_node_on_edge(canvas, cx, edge, invoked_at, candidate);
            true
        }
        _ => false,
    }
}

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

    let outcome = canvas
        .plan_canvas_split_edge_insert_candidate(cx.app, edge, &candidate, invoked_at)
        .map(|result| match result {
            Ok(ops) => Outcome::Apply(ops),
            Err(diags) => {
                let (sev, msg) = NodeGraphCanvasWith::<M>::split_edge_candidate_rejection_toast(
                    &candidate, &diags,
                );
                Outcome::Reject(sev, msg)
            }
        })
        .unwrap_or(Outcome::Ignore);

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
