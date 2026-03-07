use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_move<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        buttons: fret_core::MouseButtons,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        self.interaction.last_modifiers = modifiers;
        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);

        if pointer_move_release::handle_missing_pan_release(self, cx, position, buttons, modifiers)
        {
            return;
        }

        if pointer_move_release::handle_pending_right_click_pan_start(
            self, cx, snapshot, position, buttons, zoom,
        ) {
            return;
        }

        if pointer_move_release::handle_missing_left_release(self, cx, position, buttons, modifiers)
        {
            return;
        }

        if pointer_move_release::seed_or_update_last_pointer_state(self, position, modifiers) {
            return;
        }

        cursor::update_cursors(self, cx, snapshot, position, zoom);

        pointer_move_dispatch::dispatch_pointer_move_handlers(
            self, cx, snapshot, position, buttons, modifiers, zoom,
        );

        let snapshot = self.sync_view_state(cx.app);
        self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
    }
}
