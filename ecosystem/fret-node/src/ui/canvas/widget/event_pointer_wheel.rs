use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_wheel<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: Point,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        super::event_pointer_wheel_state::sync_pointer_wheel_modifier_state(
            self, snapshot, modifiers,
        );
        super::event_pointer_wheel_route::route_pointer_wheel(
            self, cx, snapshot, position, delta, modifiers, zoom,
        );
    }

    pub(super) fn handle_pinch_gesture<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: f32,
    ) {
        super::event_pointer_wheel_route::route_pinch_gesture(self, cx, snapshot, position, delta);
    }
}
