mod picker;
mod reroute;

use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_open_split_edge_insert_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    picker::cmd_open_split_edge_insert_node(canvas, cx, snapshot)
}

pub(super) fn cmd_insert_reroute<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    reroute::cmd_insert_reroute(canvas, cx, snapshot)
}
