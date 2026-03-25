mod apply;
mod collect;

use super::*;

pub(super) fn cmd_cut<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    canvas.copy_selection_to_clipboard(
        cx.app,
        cx.window,
        &snapshot.selected_nodes,
        &snapshot.selected_groups,
    );
    let remove_ops = collect::collect_selection_remove_ops(canvas, cx.app, snapshot);
    if remove_ops.is_empty() {
        return true;
    }

    apply::apply_remove_ops(canvas, cx, "Cut", remove_ops);
    true
}

pub(super) fn cmd_delete_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    let preferred_focus = canvas
        .interaction
        .focused_edge
        .or_else(|| snapshot.selected_edges.first().copied());
    if snapshot.selected_edges.is_empty()
        && snapshot.selected_nodes.is_empty()
        && snapshot.selected_groups.is_empty()
    {
        return true;
    }

    let remove_ops = collect::collect_selection_remove_ops(canvas, cx.app, snapshot);
    if remove_ops.is_empty() {
        return true;
    }

    apply::apply_remove_ops(canvas, cx, "Delete Selection", remove_ops);
    canvas.repair_focused_edge_after_graph_change(cx.app, preferred_focus);
    true
}
