use crate::ui::canvas::widget::*;

pub(super) fn dispatch_node_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    pending_drag::handle_pending_node_drag_move(canvas, cx, snapshot, position, zoom)
        || pending_resize::handle_pending_node_resize_move(canvas, cx, snapshot, position, zoom)
}
