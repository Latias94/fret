use super::super::*;
use super::finish_command_paint;

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
