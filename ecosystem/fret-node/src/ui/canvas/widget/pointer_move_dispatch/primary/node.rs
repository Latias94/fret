use crate::ui::canvas::widget::*;

pub(super) fn dispatch_node_move_handlers<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: pending_node_drag_activation_cx::PendingNodeDragActivationCx<H>,
{
    pending_drag::handle_pending_node_drag_move(canvas, cx, snapshot, position, zoom)
        || pending_resize::handle_pending_node_resize_move(canvas, snapshot, position, zoom)
}
