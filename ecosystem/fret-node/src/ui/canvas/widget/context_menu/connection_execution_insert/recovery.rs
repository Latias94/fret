use crate::ui::canvas::widget::*;

pub(super) fn resume_connection_insert_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::ConnectionInsertMenuCx<H>,
    fallback_from: PortId,
    invoked_at: Point,
    continue_from: Option<PortId>,
) {
    cx.resume_connection_insert_wire_drag(canvas, fallback_from, invoked_at, continue_from);
}

pub(super) fn restore_connection_menu_wire_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::ConnectionInsertMenuCx<H>,
    fallback_from: PortId,
    invoked_at: Point,
) {
    cx.restore_connection_menu_wire_drag(canvas, fallback_from, invoked_at);
}
