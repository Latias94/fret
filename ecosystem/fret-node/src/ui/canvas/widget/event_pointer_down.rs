use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_pointer_down<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        snapshot: &ViewSnapshot,
        position: Point,
        button: MouseButton,
        modifiers: fret_core::Modifiers,
        click_count: u8,
        zoom: f32,
    ) {
        if self.interaction.viewport_animation.is_some() {
            self.stop_viewport_animation_timer(cx.app);
        }
        if self.interaction.pan_inertia.is_some() {
            self.stop_pan_inertia_timer(cx.app);
            self.emit_move_end(
                &snapshot,
                ViewportMoveKind::PanInertia,
                ViewportMoveEndOutcome::Ended,
            );
        }
        self.interaction.last_pos = Some(position);
        self.interaction.last_modifiers = modifiers;
        self.interaction.multi_selection_active = snapshot
            .interaction
            .multi_selection_key
            .is_pressed(modifiers);
        self.interaction.last_canvas_pos = Some(CanvasPoint {
            x: position.x.0,
            y: position.y.0,
        });

        if searcher::handle_searcher_pointer_down(self, cx, position, button, zoom) {
            return;
        }

        if pointer_down_gesture_start::handle_close_button_pointer_down(
            self, cx, &snapshot, position, button, zoom,
        ) {
            return;
        }

        if button == MouseButton::Left
            && pointer_down_double_click::handle_background_zoom_double_click(
                self,
                cx,
                &snapshot,
                position,
                modifiers,
                click_count,
                zoom,
            )
        {
            return;
        }

        if button == MouseButton::Left
            && pointer_down_double_click::handle_edge_insert_picker_double_click(
                self,
                cx,
                &snapshot,
                position,
                modifiers,
                click_count,
                zoom,
            )
        {
            return;
        }

        if button == MouseButton::Left
            && pointer_down_double_click::handle_edge_reroute_double_click(
                self,
                cx,
                &snapshot,
                position,
                click_count,
                zoom,
            )
        {
            return;
        }

        if pointer_down_gesture_start::handle_context_menu_pointer_down(
            self, cx, position, button, zoom,
        ) {
            return;
        }

        if pointer_down_gesture_start::handle_pending_right_click_start(
            self, cx, &snapshot, position, button,
        ) {
            return;
        }

        if pointer_down_gesture_start::handle_sticky_wire_pointer_down(
            self, cx, &snapshot, position, button, zoom,
        ) {
            return;
        }

        if pointer_down_gesture_start::handle_pan_start_pointer_down(
            self, cx, &snapshot, position, button, modifiers,
        ) {
            return;
        }

        if button == MouseButton::Right
            && right_click::handle_right_click_pointer_down(self, cx, &snapshot, position, zoom)
        {
            return;
        }

        if button != MouseButton::Left {
            return;
        }

        let _ = left_click::handle_left_click_pointer_down(
            self, cx, &snapshot, position, modifiers, zoom,
        );
    }
}
