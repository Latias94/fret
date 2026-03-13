mod cursor;
mod timer;

use super::*;

pub(super) fn dispatch_pointer_move_tail<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    buttons: fret_core::MouseButtons,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) {
    cursor::update_pointer_move_cursors(canvas, cx, snapshot, position, zoom);

    pointer_move_dispatch::dispatch_pointer_move_handlers(
        canvas, cx, snapshot, position, buttons, modifiers, zoom,
    );

    timer::sync_pointer_move_auto_pan_timer(canvas, cx);
}
