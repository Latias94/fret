use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_event<H: UiHost>(
        &mut self,
        cx: &mut impl event_router_cx::EventRouteCx<H, M>,
        event: &Event,
        snapshot: &ViewSnapshot,
    ) {
        let zoom = snapshot.zoom;

        if super::event_router_system::route_non_pointer_event(self, cx, event, snapshot, zoom) {
            return;
        }

        let _ = super::event_router_pointer::route_pointer_event(self, cx, event, snapshot, zoom);
    }
}
