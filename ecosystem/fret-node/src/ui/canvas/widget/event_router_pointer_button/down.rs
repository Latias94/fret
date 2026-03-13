use super::*;

pub(super) fn route_pointer_down_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    let Event::Pointer(fret_core::PointerEvent::Down {
        position,
        button,
        modifiers,
        click_count,
        ..
    }) = event
    else {
        return false;
    };

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
