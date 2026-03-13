mod connection;
mod insert;
mod node;

use crate::ui::canvas::widget::*;

pub(super) fn dispatch_secondary_pointer_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    node::dispatch_node_move_handlers(canvas, cx, snapshot, position, modifiers, zoom)
        || connection::dispatch_connection_move_handlers(
            canvas, cx, snapshot, position, modifiers, zoom,
        )
        || insert::dispatch_insert_move_handlers(canvas, cx, snapshot, position, buttons, zoom)
}
