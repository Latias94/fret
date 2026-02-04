use super::prelude::*;

pub(super) fn handle_edge_drag_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(drag) = canvas.interaction.edge_drag.clone() else {
        return false;
    };

    let threshold_screen = snapshot.interaction.connection_drag_threshold;
    if !exceeds_drag_threshold(drag.start_pos, position, threshold_screen, zoom) {
        return false;
    }

    let geom = canvas.canvas_geometry(&*cx.app, snapshot);
    let reconnect = canvas
        .graph
        .read_ref(cx.app, |graph| {
            canvas.pick_reconnect_endpoint(
                graph,
                geom.as_ref(),
                drag.edge,
                drag.start_pos,
                snapshot.interaction.reconnect_radius,
                zoom,
            )
        })
        .ok()
        .flatten();

    let Some((endpoint, fixed)) = reconnect else {
        return false;
    };
    let endpoint_allowed = canvas
        .graph
        .read_ref(cx.app, |graph| {
            NodeGraphCanvasWith::<M>::edge_endpoint_is_reconnectable(
                graph,
                &snapshot.interaction,
                drag.edge,
                endpoint,
            )
        })
        .ok()
        .unwrap_or(false);
    if !endpoint_allowed {
        canvas.interaction.edge_drag = None;
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        return true;
    }

    canvas.interaction.edge_drag = None;
    canvas.interaction.hover_edge = None;
    let kind = WireDragKind::Reconnect {
        edge: drag.edge,
        endpoint,
        fixed,
    };
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: kind.clone(),
        pos: position,
    });
    canvas.emit_connect_start(snapshot, &kind);

    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
