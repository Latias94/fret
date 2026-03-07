use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_wheel<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: Point,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        pointer_wheel_viewport::stop_scroll_viewport_motion(self, cx, snapshot);
        self.interaction.last_modifiers = modifiers;
        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);
        if searcher::handle_searcher_wheel(self, cx, delta, modifiers, zoom) {
            return;
        }

        if pointer_wheel_viewport::handle_scroll_zoom(
            self, cx, snapshot, position, delta, modifiers, zoom,
        ) {
            return;
        }

        let _ = pointer_wheel_viewport::handle_scroll_pan(self, cx, snapshot, delta, modifiers);
    }

    pub(super) fn handle_pinch_gesture<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        delta: f32,
    ) {
        pointer_wheel_viewport::stop_pinch_viewport_motion(self, cx, snapshot);
        let _ = pointer_wheel_viewport::handle_pinch_zoom(self, cx, snapshot, position, delta);
    }
}
