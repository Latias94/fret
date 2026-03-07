use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_timer<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        token: fret_core::TimerToken,
    ) {
        if self
            .interaction
            .toast
            .as_ref()
            .is_some_and(|t| t.timer == token)
        {
            self.interaction.toast = None;
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
            return;
        }

        if timer_motion::handle_pan_inertia_tick(self, cx, snapshot, token) {
            return;
        }

        if timer_motion::handle_viewport_animation_tick(self, cx, token) {
            return;
        }

        if timer_motion::handle_auto_pan_tick(self, cx, snapshot, token) {
            return;
        }

        let _ = timer_motion::handle_viewport_move_debounce(self, cx, token);
    }
}
