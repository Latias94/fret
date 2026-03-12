use crate::ui::canvas::widget::*;

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
