use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_timer<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        token: fret_core::TimerToken,
    ) {
        if super::event_timer_toast::clear_expired_toast(self, cx, token) {
            return;
        }

        super::event_timer_route::route_timer_tick(self, cx, snapshot, token);
    }
}
