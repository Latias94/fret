use super::*;

pub(super) fn route_button_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    match event {
        Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            ..
        }) => {
            canvas.handle_pointer_down(
                cx,
                snapshot,
                *position,
                *button,
                *modifiers,
                *click_count,
                zoom,
            );
            true
        }
        Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            ..
        }) => {
            canvas.handle_pointer_move(cx, snapshot, *position, *buttons, *modifiers, zoom);
            true
        }
        Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            click_count,
            ..
        }) => {
            canvas.handle_pointer_up(
                cx,
                snapshot,
                *position,
                *button,
                *click_count,
                *modifiers,
                zoom,
            );
            true
        }
        _ => false,
    }
}
