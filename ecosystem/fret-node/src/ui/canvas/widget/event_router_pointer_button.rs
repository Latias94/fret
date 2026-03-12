mod down;
mod move_event;
mod up;

use super::*;

pub(super) fn route_button_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    match event {
        Event::Pointer(fret_core::PointerEvent::Down { .. }) => {
            down::route_pointer_down_event(canvas, cx, event, snapshot, zoom)
        }
        Event::Pointer(fret_core::PointerEvent::Move { .. }) => {
            move_event::route_pointer_move_event(canvas, cx, event, snapshot, zoom)
        }
        Event::Pointer(fret_core::PointerEvent::Up { .. }) => {
            up::route_pointer_up_event(canvas, cx, event, snapshot, zoom)
        }
        _ => false,
    }
}
