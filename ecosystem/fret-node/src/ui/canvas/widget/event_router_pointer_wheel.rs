use super::*;

pub(super) fn route_wheel_pointer_event<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    event: &Event,
    snapshot: &ViewSnapshot,
    zoom: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: super::event_router_pointer_wheel_cx::PointerWheelRouteCx<H, M>,
{
    let platform = cx.platform();
    match event {
        Event::Pointer(fret_core::PointerEvent::Wheel {
            position,
            delta,
            modifiers,
            ..
        }) => {
            canvas
                .handle_pointer_wheel(cx, platform, snapshot, *position, *delta, *modifiers, zoom);
            true
        }
        Event::Pointer(fret_core::PointerEvent::PinchGesture {
            position, delta, ..
        }) => {
            canvas.handle_pinch_gesture(cx, snapshot, *position, *delta);
            true
        }
        _ => false,
    }
}
