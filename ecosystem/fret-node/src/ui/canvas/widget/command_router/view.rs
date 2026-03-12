use super::super::*;
use super::dispatch::DirectCommandRoute;

pub(super) fn handle_direct_view_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    route: DirectCommandRoute,
) -> bool {
    match route {
        DirectCommandRoute::FrameSelection => canvas.cmd_frame_selection(cx, snapshot),
        DirectCommandRoute::FrameAll => canvas.cmd_frame_all(cx, snapshot),
        DirectCommandRoute::ResetView => canvas.cmd_reset_view(cx),
        DirectCommandRoute::ZoomIn => canvas.cmd_zoom_in(cx, snapshot),
        DirectCommandRoute::ZoomOut => canvas.cmd_zoom_out(cx, snapshot),
        DirectCommandRoute::ToggleConnectionMode => canvas.cmd_toggle_connection_mode(cx, snapshot),
        _ => false,
    }
}
