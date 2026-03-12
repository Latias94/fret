mod activation;
mod port;

use super::*;

pub(super) fn port_center_canvas<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Option<CanvasPoint> {
    port::port_center_canvas(canvas, host, snapshot, port)
}

pub(super) fn focused_port_activation_point<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Point {
    let port_center = port::port_center_canvas(canvas, host, snapshot, port)
        .map(|point| Point::new(Px(point.x), Px(point.y)));
    activation::resolve_activation_point(
        port_center,
        canvas.interaction.last_pos,
        canvas.interaction.last_bounds,
    )
}
