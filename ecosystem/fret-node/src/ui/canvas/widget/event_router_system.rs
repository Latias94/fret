use super::*;

pub(in crate::ui::canvas::widget) trait SystemRouteCx<H: UiHost, M: NodeGraphCanvasMiddleware>:
    super::event_router_system_lifecycle::SystemLifecycleCx<H>
    + super::event_keyboard::KeyboardInputSink<H, M>
{
}

impl<H: UiHost, M, T> SystemRouteCx<H, M> for T
where
    M: NodeGraphCanvasMiddleware,
    T: super::event_router_system_lifecycle::SystemLifecycleCx<H>
        + super::event_keyboard::KeyboardInputSink<H, M>,
{
}

pub(super) fn route_non_pointer_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SystemRouteCx<H, M>,
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
