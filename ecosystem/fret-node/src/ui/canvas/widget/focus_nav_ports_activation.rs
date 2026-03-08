use super::*;

pub(super) fn activate_focused_port<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut CommandCx<'_, H>,
    snapshot: &ViewSnapshot,
) -> bool {
    if !snapshot.interaction.elements_selectable {
        return false;
    }

    let Some(port) = canvas
        .interaction
        .focused_port
        .or(canvas.interaction.hover_port)
    else {
        return false;
    };

    let position = super::focus_nav_ports_center::focused_port_activation_point(
        canvas, cx.app, snapshot, port,
    );

    if canvas.interaction.wire_drag.is_none() {
        arm_click_connect_wire_drag(canvas, port, position);
        return true;
    }

    sync_wire_drag_position(canvas, position);
    let _ = wire_drag::handle_wire_left_up_with_forced_target(
        canvas,
        cx,
        snapshot,
        snapshot.zoom,
        Some(port),
    );
    super::focus_nav_ports_hints::refresh_focused_port_hints(canvas, cx.app);
    true
}

fn arm_click_connect_wire_drag<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    port: PortId,
    position: Point,
) {
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: port,
            bundle: Vec::new(),
        },
        pos: position,
    });
    canvas.interaction.click_connect = true;
    canvas.interaction.pending_wire_drag = None;
    canvas.interaction.suspended_wire_drag = None;
    canvas.interaction.sticky_wire = false;
    canvas.interaction.sticky_wire_ignore_next_up = false;
    canvas.interaction.focused_edge = None;
    canvas.interaction.focused_port = None;
    canvas.interaction.focused_port_valid = false;
    canvas.interaction.focused_port_convertible = false;
    canvas.interaction.hover_port = None;
    canvas.interaction.hover_port_valid = false;
    canvas.interaction.hover_port_convertible = false;
    canvas.interaction.hover_port_diagnostic = None;
}

fn sync_wire_drag_position<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
) {
    if let Some(mut wire_drag) = canvas.interaction.wire_drag.take() {
        wire_drag.pos = position;
        canvas.interaction.wire_drag = Some(wire_drag);
    }
}
