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

        if button == MouseButton::Right
            && snapshot.interaction.pan_on_drag.right
            && let Some(pending) = self.interaction.pending_right_click.take()
        {
            let click_distance = snapshot.interaction.pane_click_distance.max(0.0);
            let threshold = canvas_units_from_screen_px(click_distance, zoom);
            let dx = position.x.0 - pending.start_pos.x.0;
            let dy = position.y.0 - pending.start_pos.y.0;
            let is_click = click_distance == 0.0 || (dx * dx + dy * dy) <= threshold * threshold;

            cx.release_pointer_capture();
            if is_click {
                right_click::handle_right_click_pointer_down(self, cx, snapshot, position, zoom);
            }
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
