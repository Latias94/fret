use super::*;

pub(super) fn route_input_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::event_keyboard::KeyboardInputSink<H, M>,
    event: &Event,
    snapshot: &ViewSnapshot,
) -> bool {
    match event {
        Event::KeyDown { key, modifiers, .. } => {
            canvas.handle_key_down(cx, snapshot, *key, *modifiers);
            true
        }
        Event::KeyUp { key, .. } => {
            canvas.handle_key_up(cx, snapshot, *key);
            true
        }
        _ => false,
    }
}
