use super::prelude::*;

pub(super) fn handle_enter_over<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    event: &InternalDragEvent,
    payload: &super::InsertNodeDragPayload,
    zoom: f32,
) -> bool {
    let pos = event.position;
    let at = CanvasPoint {
        x: pos.x.0,
        y: pos.y.0,
    };

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

    let can_split_edge: Option<EdgeId> = edge_hit.and_then(|edge_id| {
        let candidate = &payload.candidate;
        let at = if candidate.kind.0 == REROUTE_KIND {
            canvas.reroute_pos_for_invoked_at(pos)
        } else {
            at
        };
        canvas
            .graph
            .read_ref(cx.app, |graph| {
                let presenter = &mut *canvas.presenter;
                let plan = presenter.plan_split_edge_candidate(graph, edge_id, candidate, at);
                matches!(plan.decision, ConnectDecision::Accept).then_some(edge_id)
            })
            .ok()
            .flatten()
    });

    canvas.interaction.insert_node_drag_preview = Some(InsertNodeDragPreview {
        label: payload.candidate.label.clone(),
        pos,
        edge: can_split_edge,
    });

    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    cx.stop_propagation();
    true
}
