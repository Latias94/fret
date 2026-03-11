use super::super::*;

pub(in super::super) fn collect_selection_remove_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
) -> Vec<GraphOp> {
    let selected_nodes = snapshot.selected_nodes.clone();
    let selected_edges = snapshot.selected_edges.clone();
    let selected_groups = snapshot.selected_groups.clone();
    canvas
        .graph
        .read_ref(host, |graph| {
            NodeGraphCanvasWith::<M>::delete_selection_ops(
                graph,
                &snapshot.interaction,
                &selected_nodes,
                &selected_edges,
                &selected_groups,
            )
        })
        .ok()
        .unwrap_or_default()
}
