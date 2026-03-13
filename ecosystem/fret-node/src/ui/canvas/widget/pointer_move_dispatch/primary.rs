mod connection;
mod group;
mod node;
mod surface;

use crate::ui::canvas::widget::*;

pub(super) fn dispatch_primary_pointer_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    surface::dispatch_surface_move_handlers(canvas, cx, snapshot, position, modifiers, zoom)
        || group::dispatch_group_move_handlers(canvas, cx, snapshot, position, modifiers, zoom)
        || node::dispatch_node_move_handlers(canvas, cx, snapshot, position, zoom)
        || connection::dispatch_connection_move_handlers(
            canvas, cx, snapshot, position, modifiers, zoom,
        )
}
