use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_key_down<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
        modifiers: fret_core::Modifiers,
    ) {
        if super::event_keyboard_state::should_ignore_key_down(cx.input_ctx.focus_is_text_input) {
            return;
        }

        super::event_keyboard_state::sync_keyboard_modifier_state(self, snapshot, modifiers);
        super::event_keyboard_route::route_key_down(self, cx, snapshot, key, modifiers);
    }

    pub(super) fn handle_key_up<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
    ) {
        super::event_keyboard_route::route_key_up(self, cx, snapshot, key);
    }
}
