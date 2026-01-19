use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn cmd_toggle_connection_mode<H: UiHost>(
        &mut self,
        cx: &mut CommandCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> bool {
        let next = match snapshot.interaction.connection_mode {
            NodeGraphConnectionMode::Strict => NodeGraphConnectionMode::Loose,
            NodeGraphConnectionMode::Loose => NodeGraphConnectionMode::Strict,
        };

        self.update_view_state(cx.app, |s| {
            s.interaction.connection_mode = next;
        });
        self.show_toast(
            cx.app,
            cx.window,
            DiagnosticSeverity::Info,
            match next {
                NodeGraphConnectionMode::Strict => "connection mode: strict",
                NodeGraphConnectionMode::Loose => "connection mode: loose",
            },
        );
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
        true
    }
}
