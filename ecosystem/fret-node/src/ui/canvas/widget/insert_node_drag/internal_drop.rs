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
        let at = if candidate.kind.0 == REROUTE_KIND {
            canvas.reroute_pos_for_invoked_at(pos)
        } else {
            at
        };
        let planned = canvas
            .graph
            .read_ref(cx.app, |graph| {
                let presenter = &mut *canvas.presenter;
                let plan = presenter.plan_split_edge_candidate(graph, edge_id, &candidate, at);
                match plan.decision {
                    ConnectDecision::Accept => Some(Ok(plan.ops)),
                    ConnectDecision::Reject => Some(Err(plan.diagnostics)),
                }
            })
            .ok()
            .flatten();

        if let Some(Ok(ops)) = planned {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            applied = canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops);
            if applied && let Some(node_id) = node_id {
                canvas.update_view_state(cx.app, |s| {
                    s.selected_edges.clear();
                    s.selected_groups.clear();
                    s.selected_nodes.clear();
                    s.selected_nodes.push(node_id);
                    s.draw_order.retain(|id| *id != node_id);
                    s.draw_order.push(node_id);
                });
            }
        } else if let Some(Err(diags)) = planned {
            if let Some((sev, msg)) = NodeGraphCanvasWith::<M>::toast_from_diagnostics(&diags) {
                canvas.show_toast(cx.app, cx.window, sev, msg);
            }
        }
    }

    if !applied {
        let ops: Option<Vec<GraphOp>> = if candidate.kind.0 == REROUTE_KIND {
            Some(NodeGraphCanvasWith::<M>::build_reroute_create_ops(at))
        } else {
            let presenter = &mut *canvas.presenter;
            canvas
                .graph
                .read_ref(cx.app, |graph| {
                    presenter.plan_create_node(graph, &candidate, at)
                })
                .ok()
                .and_then(|r| r.ok())
        };

        if let Some(ops) = ops {
            let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
            if canvas.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
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
        } else {
            canvas.show_toast(
                cx.app,
                cx.window,
                crate::rules::DiagnosticSeverity::Info,
                Arc::<str>::from("node insertion is not supported"),
            );
        }
    }

    canvas.interaction.insert_node_drag_preview = None;
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    cx.stop_propagation();
    true
}
