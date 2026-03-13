use crate::ui::canvas::widget::*;

pub(super) fn arm_click_connect_wire_drag<M: NodeGraphCanvasMiddleware>(
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
    super::super::focus_session::clear_focused_port_hints(&mut canvas.interaction);
    super::super::focus_session::clear_hover_port_hints(&mut canvas.interaction);
}
