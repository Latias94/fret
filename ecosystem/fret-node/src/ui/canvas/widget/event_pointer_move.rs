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
        super::event_pointer_move_state::sync_pointer_move_modifier_state(
            self, snapshot, modifiers,
        );

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

        if super::event_pointer_move_state::seed_or_update_last_pointer_state(
            self, position, modifiers,
        ) {
            return;
        }

        super::event_pointer_move_tail::dispatch_pointer_move_tail(
            self, cx, snapshot, position, buttons, modifiers, zoom,
        );
    }
}
