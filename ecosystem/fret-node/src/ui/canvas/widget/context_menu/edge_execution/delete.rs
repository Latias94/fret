use crate::ui::canvas::widget::*;

pub(super) fn delete_edge<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::EdgeContextActionCx<H>,
    edge_id: EdgeId,
) {
    let remove_ops = {
        let this = &*canvas;
        this.graph
            .read_ref(cx.host(), |graph| {
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
    let window = cx.window();
    canvas.apply_ops(cx.host(), window, remove_ops);
    canvas.update_view_state(cx.host(), |view_state| {
        view_state.selected_edges.retain(|id| *id != edge_id);
    });
}
