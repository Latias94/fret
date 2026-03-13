use super::*;

pub(super) fn apply_directional_port_focus<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    next_port: PortId,
) -> bool {
    let Some(owner) = focused_port_owner(canvas, host, next_port) else {
        return false;
    };

    super::focus_session::focus_port(&mut canvas.interaction, owner, next_port);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |state| {
        super::focus_session::select_only_node(state, owner, true);
    });

    let snapshot = canvas.sync_view_state(host);
    if let Some(center) = canvas.port_center_canvas(host, &snapshot, next_port) {
        canvas.ensure_canvas_point_visible(host, &snapshot, center);
    }
    true
}

fn focused_port_owner<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    port: PortId,
) -> Option<GraphNodeId> {
    canvas
        .graph
        .read_ref(host, |graph| graph.ports.get(&port).map(|port| port.node))
        .ok()
        .flatten()
}
