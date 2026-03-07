use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_up<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        button: MouseButton,
        click_count: u8,
        modifiers: fret_core::Modifiers,
        zoom: f32,
    ) {
        self.interaction.last_modifiers = modifiers;
        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);

        if right_click::handle_pending_right_click_pointer_up(
            self, cx, snapshot, position, button, zoom,
        ) {
            return;
        }

        if button == MouseButton::Left
            && searcher::handle_searcher_pointer_up(self, cx, position, button, zoom)
        {
            return;
        }

        let _ = pointer_up::handle_pointer_up(
            self,
            cx,
            snapshot,
            position,
            button,
            click_count,
            modifiers,
            zoom,
        );
    }
}
