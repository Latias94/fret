use super::*;

pub(super) fn refresh_focused_port_hints<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    super::focus_nav_ports_hints::refresh_focused_port_hints(canvas, host)
}

pub(super) fn port_center_canvas<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Option<CanvasPoint> {
    super::focus_nav_ports_center::port_center_canvas(canvas, host, snapshot, port)
}

pub(super) fn activate_focused_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    super::focus_nav_ports_activation::activate_focused_port(canvas, cx, snapshot)
}
