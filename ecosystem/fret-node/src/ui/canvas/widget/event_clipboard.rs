use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_clipboard_text<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        token: fret_core::ClipboardToken,
        text: &str,
    ) {
        let Some(pending) = super::event_clipboard_pending::take_matching_pending_paste(
            &mut self.interaction.pending_paste,
            token,
        ) else {
            return;
        };

        self.apply_paste_text(cx.app, cx.window, text, pending.at);
        super::event_clipboard_feedback::request_paste_feedback(cx);
    }

    pub(super) fn handle_clipboard_text_unavailable<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        token: fret_core::ClipboardToken,
    ) {
        if !super::event_clipboard_pending::clear_pending_if_matches(
            &mut self.interaction.pending_paste,
            token,
        ) {
            return;
        }

        super::event_clipboard_feedback::show_clipboard_unavailable_toast(self, cx);
    }
}
