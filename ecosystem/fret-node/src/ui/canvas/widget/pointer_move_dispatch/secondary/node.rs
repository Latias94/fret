use crate::ui::canvas::widget::*;

pub(super) fn dispatch_node_move_handlers<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: node_resize_move_cx::NodeResizeMoveCx<H> + node_drag_move_cx::NodeDragMoveCx<H>,
{
    node_resize::handle_node_resize_move(canvas, cx, snapshot, position, modifiers, zoom)
        || node_drag::handle_node_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
