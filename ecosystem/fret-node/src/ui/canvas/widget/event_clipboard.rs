use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_clipboard_text<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        token: fret_core::ClipboardToken,
        text: &str,
    ) {
        let Some(pending) = self.interaction.pending_paste.take() else {
            return;
        };
        if pending.token != token {
            self.interaction.pending_paste = Some(pending);
            return;
        }
        self.apply_paste_text(cx.app, cx.window, text, pending.at);
        cx.request_redraw();
        cx.invalidate_self(Invalidation::Paint);
    }

    pub(super) fn handle_clipboard_text_unavailable<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        token: fret_core::ClipboardToken,
    ) {
        if let Some(pending) = &self.interaction.pending_paste
            && pending.token == token
        {
            self.interaction.pending_paste = None;
            self.show_toast(
                cx.app,
                cx.window,
                DiagnosticSeverity::Info,
                "clipboard text unavailable",
            );
            cx.request_redraw();
            cx.invalidate_self(Invalidation::Paint);
        }
    }
}
