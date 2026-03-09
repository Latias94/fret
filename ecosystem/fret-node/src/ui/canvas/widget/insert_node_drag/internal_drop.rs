use super::prelude::*;

pub(super) fn handle_drop<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    event: &InternalDragEvent,
    payload: super::InsertNodeDragPayload,
    zoom: f32,
) -> bool {
    let pos = event.position;
    let at = CanvasPoint {
        x: pos.x.0,
        y: pos.y.0,
    };
    let candidate = payload.candidate;
    canvas.record_recent_kind(&candidate.kind);

    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let edge_hit: Option<EdgeId> = canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
            canvas.hit_edge(graph, snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten();

    let mut applied = false;

    if let Some(edge_id) = edge_hit {
        let planned =
            canvas.plan_canvas_split_edge_insert_candidate(cx.app, edge_id, &candidate, pos);

        if let Some(Ok(ops)) = planned {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            applied = canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops);
            if applied {
                canvas.select_inserted_node(cx.app, node_id);
            }
        } else if let Some(Err(diags)) = planned {
            let (sev, msg) =
                NodeGraphCanvasWith::<M>::split_edge_candidate_rejection_toast(&candidate, &diags);
            canvas.show_toast(cx.app, cx.window, sev, msg);
        }
    }

    if !applied {
        let ops = canvas
            .plan_canvas_insert_candidate_ops(cx.app, &candidate, at)
            .and_then(|result| result.ok());

        if let Some(ops) = ops {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            if canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                canvas.select_inserted_node(cx.app, node_id);
            }
        } else {
            canvas.show_toast(
                cx.app,
                cx.window,
                crate::rules::DiagnosticSeverity::Info,
                Arc::<str>::from("node insertion is not supported"),
            );
        }
    }

    super::session::clear_insert_node_drag_preview(&mut canvas.interaction, cx);
    super::session::finish_insert_node_drag_event(cx)
}
