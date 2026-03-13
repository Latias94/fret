use crate::ui::canvas::widget::*;

pub(super) fn resume_connection_insert_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    fallback_from: PortId,
    invoked_at: Point,
    continue_from: Option<PortId>,
) {
    let resume_pos = canvas.interaction.last_pos.unwrap_or(invoked_at);
    if let Some(port) = continue_from {
        canvas.interaction.suspended_wire_drag = None;
        canvas.start_sticky_wire_drag_from_port(cx, port, resume_pos);
    } else {
        canvas.restore_suspended_wire_drag(cx, Some(fallback_from), resume_pos);
    }
}

pub(super) fn restore_connection_menu_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    fallback_from: PortId,
    invoked_at: Point,
) {
    let resume_pos = canvas.interaction.last_pos.unwrap_or(invoked_at);
    canvas.restore_suspended_wire_drag(cx, Some(fallback_from), resume_pos);
}
