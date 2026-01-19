use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_nudge_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        delta: CanvasPoint,
    ) -> bool {
        self.nudge_selection_by_screen_delta(cx.app, cx.window, snapshot, delta);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }

    pub(super) fn cmd_align_or_distribute_selection<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
        mode: AlignDistributeMode,
    ) -> bool {
        self.align_or_distribute_selection(cx.app, cx.window, snapshot, mode);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }
}
