use crate::ui::canvas::widget::*;

pub(super) fn delete_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge_id: EdgeId,
) {
    let remove_ops = {
        let this = &*canvas;
        this.graph
            .read_ref(cx.app, |graph| {
                graph
                    .edges
                    .get(&edge_id)
                    .map(|edge| {
                        vec![GraphOp::RemoveEdge {
                            id: edge_id,
                            edge: edge.clone(),
                        }]
                    })
                    .unwrap_or_default()
            })
            .ok()
            .unwrap_or_default()
    };
    canvas.apply_ops(cx.app, cx.window, remove_ops);
    canvas.update_view_state(cx.app, |view_state| {
        view_state.selected_edges.retain(|id| *id != edge_id);
    });
}
