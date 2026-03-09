use super::*;

pub(super) fn route_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    if super::event_router_pointer_button::route_button_pointer_event(
        canvas, cx, event, snapshot, zoom,
    ) {
        return true;
    }

    super::event_router_pointer_wheel::route_wheel_pointer_event(canvas, cx, event, snapshot, zoom)
}
