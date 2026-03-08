use super::*;

pub(super) fn apply_directional_port_focus<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    next_port: PortId,
) -> bool {
    let Some(owner) = focused_port_owner(canvas, host, next_port) else {
        return false;
    };

    canvas.interaction.focused_node = Some(owner);
    canvas.interaction.focused_edge = None;
    canvas.interaction.focused_port = Some(next_port);
    canvas.refresh_focused_port_hints(host);
    canvas.update_view_state(host, |state| {
        state.selected_edges.clear();
        state.selected_groups.clear();
        state.selected_nodes.clear();
        state.selected_nodes.push(owner);
        state.draw_order.retain(|id| *id != owner);
        state.draw_order.push(owner);
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
