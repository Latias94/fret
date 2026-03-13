mod overlay;
mod primary;
mod secondary;

use super::*;

pub(super) fn dispatch_pointer_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    if primary::dispatch_primary_pointer_move_handlers(
        canvas, cx, snapshot, position, modifiers, zoom,
    ) {
    } else if secondary::dispatch_secondary_pointer_move_handlers(
        canvas, cx, snapshot, position, buttons, modifiers, zoom,
    ) {
    } else if overlay::dispatch_overlay_pointer_move_handlers(canvas, cx, snapshot, position, zoom)
    {
    } else {
        hover::update_hover_edge(canvas, cx, snapshot, position, zoom);
    }
}
