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
    super::super::event_pointer_move_tail::dispatch_pointer_move_tail(
        canvas, cx, snapshot, position, buttons, modifiers, zoom,
    );
}
