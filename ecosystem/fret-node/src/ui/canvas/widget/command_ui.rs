use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn dismiss_command_transients(&mut self) {
        self.interaction.context_menu = None;
        super::searcher_activation_state::clear_searcher_overlay(&mut self.interaction);
    }

    pub(super) fn dismiss_command_context_menu(&mut self) {
        self.interaction.context_menu = None;
    }

    pub(super) fn command_invoked_at(&self) -> Point {
        self.interaction
            .last_pos
            .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)))
    }
}

pub(super) fn finish_command_paint<H: UiHost>(cx: &mut CommandCx<'_, H>) -> bool {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
