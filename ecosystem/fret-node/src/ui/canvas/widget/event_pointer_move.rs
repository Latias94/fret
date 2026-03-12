mod pointer_state;
mod release;
mod tail;

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
        pointer_state::sync_pointer_move_modifier_state(self, snapshot, modifiers);

        if release::handle_pointer_move_release_guards(
            self, cx, snapshot, position, buttons, modifiers, zoom,
        ) {
            return;
        }

        if pointer_state::seed_or_update_last_pointer_state(self, position, modifiers) {
            return;
        }

        tail::dispatch_pointer_move_tail(self, cx, snapshot, position, buttons, modifiers, zoom);
    }
}
