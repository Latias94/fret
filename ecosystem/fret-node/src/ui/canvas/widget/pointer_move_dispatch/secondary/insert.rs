use crate::ui::canvas::widget::*;

pub(super) fn dispatch_insert_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    zoom: f32,
) -> bool {
    insert_node_drag::handle_pending_insert_node_drag_move(
        canvas, cx, snapshot, position, buttons, zoom,
    )
}
