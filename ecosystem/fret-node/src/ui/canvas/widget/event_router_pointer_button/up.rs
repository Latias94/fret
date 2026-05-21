use super::*;

pub(super) fn route_pointer_up_event<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: super::super::event_pointer_up::PointerUpRouteCx<H, M>,
{
    let Event::Pointer(fret_core::PointerEvent::Up {
        position,
        button,
        modifiers,
        click_count,
        ..
    }) = event
    else {
        return false;
    };

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
