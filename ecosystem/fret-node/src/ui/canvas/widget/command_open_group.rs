mod create;
mod order;
mod rename;

use super::command_ui::finish_command_paint;
use super::*;

pub(super) fn cmd_create_group<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
) -> bool {
    create::cmd_create_group(canvas, cx)
}

pub(super) fn cmd_group_bring_to_front<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    order::cmd_group_bring_to_front(canvas, cx, snapshot)
}

pub(super) fn cmd_group_send_to_back<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    order::cmd_group_send_to_back(canvas, cx, snapshot)
}

pub(super) fn cmd_group_rename<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    rename::cmd_group_rename(canvas, cx, snapshot)
}
