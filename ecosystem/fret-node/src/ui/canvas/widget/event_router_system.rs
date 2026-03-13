use super::*;

pub(super) fn route_non_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool {
    if super::event_router_system_lifecycle::route_lifecycle_event(
        canvas, cx, event, snapshot, zoom,
    ) {
        return true;
    }

    super::event_router_system_input::route_input_event(canvas, cx, event, snapshot)
}
