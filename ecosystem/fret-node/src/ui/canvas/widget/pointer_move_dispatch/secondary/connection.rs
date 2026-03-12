use crate::ui::canvas::widget::*;

pub(super) fn dispatch_connection_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    wire_drag::handle_wire_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
        || edge_insert_drag::handle_edge_insert_drag_move(canvas, cx, position)
        || edge_drag::handle_edge_drag_move(canvas, cx, snapshot, position, zoom)
}
