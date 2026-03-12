mod center;
mod search;

use super::*;

pub(super) fn required_port_direction_from_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
) -> Option<PortDirection> {
    super::focus_port_direction_wire::required_port_direction_from_wire_drag(canvas, host)
}

pub(super) fn directional_port_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    from_port: PortId,
    from_center: CanvasPoint,
    dir: PortNavDir,
    required_dir: Option<PortDirection>,
) -> Option<PortId> {
    search::directional_port_candidate(
        canvas,
        host,
        snapshot,
        from_port,
        from_center,
        dir,
        required_dir,
    )
}
