use crate::ui::canvas::widget::*;

pub(super) fn apply_split_edge_reroute_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    label: Option<&str>,
    ops: Vec<GraphOp>,
) -> bool {
    let node_id = NodeGraphCanvasWith::<M>::first_added_node_id(&ops);
    let applied = canvas.commit_ops(host, window, label, ops);
    if applied {
        canvas.select_inserted_node(host, node_id);
    }
    applied
}
