use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_wheel<H: UiHost, Cx>(
        &mut self,
        cx: &mut Cx,
        platform: fret_runtime::Platform,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: Point,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) where
        Cx: pointer_wheel_cx::PointerWheelCx<H, M>,
    {
        super::event_pointer_wheel_state::sync_pointer_wheel_modifier_state(
            self, snapshot, modifiers,
        );
        super::event_pointer_wheel_route::route_pointer_wheel(
            self, cx, platform, snapshot, position, delta, modifiers, zoom,
        );
    }

    pub(super) fn handle_pinch_gesture<H: UiHost, Cx>(
        &mut self,
        cx: &mut Cx,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: f32,
    ) where
        Cx: viewport_motion_cx::ViewportMotionCx<H>,
    {
        super::event_pointer_wheel_route::route_pinch_gesture(self, cx, snapshot, position, delta);
    }
}
