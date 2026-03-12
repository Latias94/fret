use super::super::*;
use super::dispatch::DirectCommandRoute;

pub(super) fn handle_direct_edit_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    route: DirectCommandRoute,
) -> bool {
    match route {
        DirectCommandRoute::Undo => canvas.cmd_undo(cx, snapshot),
        DirectCommandRoute::Redo => canvas.cmd_redo(cx, snapshot),
        DirectCommandRoute::SelectAll => canvas.cmd_select_all(cx, snapshot),
        DirectCommandRoute::Copy => canvas.cmd_copy(cx, snapshot),
        DirectCommandRoute::Cut => canvas.cmd_cut(cx, snapshot),
        DirectCommandRoute::Paste => canvas.cmd_paste(cx, snapshot),
        DirectCommandRoute::Duplicate => canvas.cmd_duplicate(cx, snapshot),
        DirectCommandRoute::DeleteSelection => canvas.cmd_delete_selection(cx, snapshot),
        _ => false,
    }
}
