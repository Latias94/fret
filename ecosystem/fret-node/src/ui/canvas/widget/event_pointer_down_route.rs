mod dispatch;
mod double_click;
mod preflight;
mod starts;

use super::*;

pub(super) fn route_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    button: MouseButton,
    modifiers: fret_core::Modifiers,
    click_count: u8,
    zoom: f32,
) {
    if preflight::handle_pointer_down_preflight(
        canvas,
        cx,
        snapshot,
        position,
        button,
        modifiers,
        click_count,
        zoom,
    ) {
        return;
    }
    if starts::handle_pointer_down_starts(canvas, cx, snapshot, position, button, modifiers, zoom) {
        return;
    }

    dispatch::dispatch_tail_pointer_down(canvas, cx, snapshot, position, button, modifiers, zoom);
}
