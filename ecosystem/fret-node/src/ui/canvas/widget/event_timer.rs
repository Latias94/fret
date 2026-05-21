use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_timer<H: UiHost, Cx>(
        &mut self,
        cx: &mut Cx,
        snapshot: &ViewSnapshot,
        token: fret_core::TimerToken,
    ) where
        Cx: timer_motion_cx::TimerMotionCx<H>,
    {
        if super::event_timer_toast::clear_expired_toast(self, cx, token) {
            return;
        }

        super::event_timer_route::route_timer_tick(self, cx, snapshot, token);
    }
}
