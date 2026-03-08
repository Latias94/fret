use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_key_down<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
        modifiers: fret_core::Modifiers,
    ) {
        if cx.input_ctx.focus_is_text_input {
            return;
        }

        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);

        if keyboard_shortcuts::handle_escape_key(self, cx, key) {
            return;
        }

        if keyboard_shortcuts::handle_overlay_key_down(self, cx, key, modifiers) {
            return;
        }

        if keyboard_shortcuts::handle_modifier_shortcuts(cx, snapshot, key, modifiers) {
            return;
        }

        if keyboard_shortcuts::handle_tab_navigation(self, cx, snapshot, key, modifiers) {
            return;
        }

        if keyboard_pan_activation::handle_pan_activation_key_down(
            self, cx, snapshot, key, modifiers,
        ) {
            return;
        }

        if keyboard_shortcuts::handle_arrow_nudging(cx, snapshot, key, modifiers) {
            return;
        }

        let _ = keyboard_shortcuts::handle_delete_shortcut(cx, snapshot, key);
    }

    pub(super) fn handle_key_up<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        key: fret_core::KeyCode,
    ) {
        let _ = keyboard_pan_activation::handle_pan_activation_key_up(self, cx, snapshot, key);
    }
}
