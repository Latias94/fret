use super::*;

pub(super) fn cmd_cut<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    canvas.copy_selection_to_clipboard(cx.app, &snapshot.selected_nodes, &snapshot.selected_groups);
    let remove_ops = collect_selection_remove_ops(canvas, cx.app, snapshot);
    if remove_ops.is_empty() {
        return true;
    }

    apply_remove_ops(canvas, cx, "Cut", remove_ops);
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

    let remove_ops = collect_selection_remove_ops(canvas, cx.app, snapshot);
    if remove_ops.is_empty() {
        return true;
    }

    apply_remove_ops(canvas, cx, "Delete Selection", remove_ops);
    canvas.repair_focused_edge_after_graph_change(cx.app, preferred_focus);
    true
}

fn collect_selection_remove_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
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

fn apply_remove_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
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
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}
