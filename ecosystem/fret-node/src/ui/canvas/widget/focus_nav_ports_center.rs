use super::*;

pub(super) fn port_center_canvas<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Option<CanvasPoint> {
    let (geom, _) = canvas.canvas_derived(&*host, snapshot);
    geom.ports.get(&port).map(|handle| CanvasPoint {
        x: handle.center.x.0,
        y: handle.center.y.0,
    })
}

pub(super) fn focused_port_activation_point<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    port: PortId,
) -> Point {
    port_center_canvas(canvas, host, snapshot, port)
        .map(|point| Point::new(Px(point.x), Px(point.y)))
        .or(canvas.interaction.last_pos)
        .unwrap_or_else(|| {
            let bounds = canvas.interaction.last_bounds.unwrap_or_default();
            Point::new(
                Px(bounds.origin.x.0 + 0.5 * bounds.size.width.0),
                Px(bounds.origin.y.0 + 0.5 * bounds.size.height.0),
            )
        })
}
