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
        canvas
            .can_split_edge_insert_candidate(cx.app, edge_id, &payload.candidate, pos)
            .and_then(|accepted| accepted.then_some(edge_id))
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
