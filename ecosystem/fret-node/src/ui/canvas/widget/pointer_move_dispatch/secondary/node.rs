use crate::ui::canvas::widget::*;

pub(super) fn dispatch_node_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    node_resize::handle_node_resize_move(canvas, cx, snapshot, position, modifiers, zoom)
        || node_drag::handle_node_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
}
