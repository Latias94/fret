use super::*;

pub(super) fn route_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Move {
        position,
        buttons,
        modifiers,
        ..
    }) = event
    else {
        return false;
    };

    canvas.handle_pointer_move(cx, snapshot, *position, *buttons, *modifiers, zoom);
    true
}
