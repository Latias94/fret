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

        if button == MouseButton::Left {
            if let Some(command) = self.close_command.clone() {
                let rect = Self::close_button_rect(snapshot.pan, zoom);
                if Self::rect_contains(rect, position) {
                    cx.dispatch_command(command);
                    cx.stop_propagation();
                    return;
                }
            }
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

        if self.interaction.context_menu.is_some()
            && context_menu::handle_context_menu_pointer_down(self, cx, position, button, zoom)
        {
            return;
        }

        if button == MouseButton::Right {
            cancel::cancel_active_gestures(self, cx);
            if snapshot.interaction.pan_on_drag.right {
                self.interaction.pending_right_click =
                    Some(crate::ui::canvas::state::PendingRightClick {
                        start_pos: position,
                    });
                cx.capture_pointer(cx.node);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                return;
            }
        }

        if sticky_wire::handle_sticky_wire_pointer_down(self, cx, &snapshot, position, button, zoom)
        {
            return;
        }

        if button == MouseButton::Left
            && snapshot.interaction.space_to_pan
            && self.interaction.pan_activation_key_held
            && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
        {
            let _ = pan_zoom::begin_panning(
                self,
                cx,
                &snapshot,
                position,
                fret_core::MouseButton::Left,
            );
            return;
        }

        if button == MouseButton::Middle && snapshot.interaction.pan_on_drag.middle {
            let _ = pan_zoom::begin_panning(
                self,
                cx,
                &snapshot,
                position,
                fret_core::MouseButton::Middle,
            );
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
