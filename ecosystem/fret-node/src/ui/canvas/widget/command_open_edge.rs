use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_open_split_edge_insert_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    if snapshot.selected_edges.len() != 1 {
        return true;
    }
    let edge_id = snapshot.selected_edges[0];
    let invoked_at = canvas.command_invoked_at();
    canvas.open_edge_insert_node_picker(cx.app, cx.window, edge_id, invoked_at);
    finish_command_paint(cx)
}

pub(super) fn cmd_insert_reroute<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    if snapshot.selected_edges.len() != 1 {
        return true;
    }
    let edge_id = snapshot.selected_edges[0];
    let invoked_at = canvas.command_invoked_at();
    let outcome = canvas.plan_canvas_split_edge_reroute(cx.app, edge_id, invoked_at);
    canvas.execute_split_edge_reroute_outcome(cx.app, cx.window, Some("Insert Reroute"), outcome);
    finish_command_paint(cx)
}
