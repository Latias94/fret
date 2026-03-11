use super::super::*;

pub(in super::super) fn apply_remove_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    label: &'static str,
    remove_ops: Vec<GraphOp>,
) {
    let (removed_nodes, removed_edges, removed_groups) =
        NodeGraphCanvasWith::<M>::removed_ids_from_ops(&remove_ops);
    let _ = canvas.commit_ops(cx.app, cx.window, Some(label), remove_ops);
    canvas.update_view_state(cx.app, |s| {
        s.selected_edges.retain(|id| !removed_edges.contains(id));
        s.selected_nodes.retain(|id| !removed_nodes.contains(id));
        s.selected_groups.retain(|id| !removed_groups.contains(id));
    });
    super::super::command_ui::finish_command_paint(cx);
}
