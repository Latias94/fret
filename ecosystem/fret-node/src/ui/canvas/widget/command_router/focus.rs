use super::super::*;
use super::dispatch::DirectCommandRoute;

pub(super) fn handle_direct_focus_command<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
    route: DirectCommandRoute,
) -> bool {
    match route {
        DirectCommandRoute::FocusNextNode => canvas.cmd_focus_next_node(cx, snapshot),
        DirectCommandRoute::FocusPrevNode => canvas.cmd_focus_prev_node(cx, snapshot),
        DirectCommandRoute::FocusNextEdge => canvas.cmd_focus_next_edge(cx, snapshot),
        DirectCommandRoute::FocusPrevEdge => canvas.cmd_focus_prev_edge(cx, snapshot),
        DirectCommandRoute::FocusNextPort => canvas.cmd_focus_next_port(cx, snapshot),
        DirectCommandRoute::FocusPrevPort => canvas.cmd_focus_prev_port(cx, snapshot),
        DirectCommandRoute::FocusPortLeft => canvas.cmd_focus_port_left(cx, snapshot),
        DirectCommandRoute::FocusPortRight => canvas.cmd_focus_port_right(cx, snapshot),
        DirectCommandRoute::FocusPortUp => canvas.cmd_focus_port_up(cx, snapshot),
        DirectCommandRoute::FocusPortDown => canvas.cmd_focus_port_down(cx, snapshot),
        DirectCommandRoute::Activate => canvas.cmd_activate(cx, snapshot),
        _ => false,
    }
}
