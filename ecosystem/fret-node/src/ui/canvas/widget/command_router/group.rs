use super::super::*;
use super::dispatch::DirectCommandRoute;

pub(super) fn handle_direct_group_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    route: DirectCommandRoute,
) -> bool {
    match route {
        DirectCommandRoute::CreateGroup => canvas.cmd_create_group(cx),
        DirectCommandRoute::GroupBringToFront => canvas.cmd_group_bring_to_front(cx, snapshot),
        DirectCommandRoute::GroupSendToBack => canvas.cmd_group_send_to_back(cx, snapshot),
        DirectCommandRoute::GroupRename => canvas.cmd_group_rename(cx, snapshot),
        _ => false,
    }
}
