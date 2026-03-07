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

        // The runtime may occasionally miss a corresponding `PointerEvent::Up` (e.g. when
        // releasing outside of the window / losing capture). Infer the release from the
        // current button state and synthesize an "up" so we can finish the interaction
        // through the canonical pointer-up code path (commit, cancel, inertia, etc.).
        if self.interaction.panning {
            let should_end = match self.interaction.panning_button {
                Some(fret_core::MouseButton::Middle) => !buttons.middle,
                Some(fret_core::MouseButton::Left) => !buttons.left,
                Some(fret_core::MouseButton::Right) => !buttons.right,
                _ => false,
            };
            if should_end {
                let snapshot = self.sync_view_state(cx.app);
                let button = self
                    .interaction
                    .panning_button
                    .unwrap_or(fret_core::MouseButton::Middle);
                let _ = pointer_up::handle_pointer_up(
                    self,
                    cx,
                    &snapshot,
                    position,
                    button,
                    1,
                    modifiers,
                    snapshot.zoom,
                );
                return;
            }
        }

        if snapshot.interaction.pan_on_drag.right
            && buttons.right
            && self.interaction.panning_button.is_none()
            && let Some(pending) = self.interaction.pending_right_click
            && right_click::pending_right_click_exceeded_drag_threshold(
                pending,
                position,
                snapshot.interaction.pane_click_distance,
                zoom,
            )
        {
            self.interaction.pending_right_click = None;
            let _ = pan_zoom::begin_panning(
                self,
                cx,
                snapshot,
                position,
                fret_core::MouseButton::Right,
            );
            return;
        }

        let has_left_interaction = self.interaction.pending_marquee.is_some()
            || self.interaction.marquee.is_some()
            || self.interaction.pending_node_drag.is_some()
            || self.interaction.node_drag.is_some()
            || self.interaction.pending_group_drag.is_some()
            || self.interaction.group_drag.is_some()
            || self.interaction.pending_group_resize.is_some()
            || self.interaction.group_resize.is_some()
            || self.interaction.pending_node_resize.is_some()
            || self.interaction.node_resize.is_some()
            || self.interaction.pending_wire_drag.is_some()
            || self.interaction.wire_drag.is_some()
            || self.interaction.pending_edge_insert_drag.is_some()
            || self.interaction.edge_insert_drag.is_some()
            || self.interaction.edge_drag.is_some();

        if has_left_interaction && !buttons.left {
            let snapshot = self.sync_view_state(cx.app);
            let _ = pointer_up::handle_pointer_up(
                self,
                cx,
                &snapshot,
                position,
                fret_core::MouseButton::Left,
                1,
                modifiers,
                snapshot.zoom,
            );
            return;
        }

        if self.interaction.last_pos.is_none() {
            self.interaction.last_pos = Some(position);
            self.interaction.last_modifiers = modifiers;
            self.interaction.last_canvas_pos = Some(CanvasPoint {
                x: position.x.0,
                y: position.y.0,
            });
            return;
        }
        self.interaction.last_pos = Some(position);
        self.interaction.last_modifiers = modifiers;
        self.interaction.last_canvas_pos = Some(CanvasPoint {
            x: position.x.0,
            y: position.y.0,
        });

        cursor::update_cursors(self, cx, snapshot, position, zoom);

        if pan_zoom::handle_panning_move(self, cx, snapshot, position) {
            // keep going to sync auto-pan timer
        } else if marquee::handle_marquee_move(self, cx, snapshot, position, modifiers, zoom) {
            // keep going to sync auto-pan timer
        } else if pending_group_drag::handle_pending_group_drag_move(
            self, cx, snapshot, position, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if group_drag::handle_group_drag_move(self, cx, snapshot, position, modifiers, zoom)
        {
            // keep going to sync auto-pan timer
        } else if pending_group_resize::handle_pending_group_resize_move(
            self, cx, snapshot, position, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if group_resize::handle_group_resize_move(
            self, cx, snapshot, position, modifiers, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if pending_drag::handle_pending_node_drag_move(self, cx, snapshot, position, zoom) {
            // keep going to sync auto-pan timer
        } else if pending_resize::handle_pending_node_resize_move(
            self, cx, snapshot, position, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if pending_wire_drag::handle_pending_wire_drag_move(
            self, cx, snapshot, position, modifiers, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if edge_insert_drag::handle_pending_edge_insert_drag_move(
            self, cx, snapshot, position,
        ) {
            // keep going to sync auto-pan timer
        } else if node_resize::handle_node_resize_move(
            self, cx, snapshot, position, modifiers, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if node_drag::handle_node_drag_move(self, cx, snapshot, position, modifiers, zoom) {
            // keep going to sync auto-pan timer
        } else if wire_drag::handle_wire_drag_move(self, cx, snapshot, position, modifiers, zoom) {
            // keep going to sync auto-pan timer
        } else if edge_insert_drag::handle_edge_insert_drag_move(self, cx, position) {
            // keep going to sync auto-pan timer
        } else if edge_drag::handle_edge_drag_move(self, cx, snapshot, position, zoom) {
            // keep going to sync auto-pan timer
        } else if insert_node_drag::handle_pending_insert_node_drag_move(
            self, cx, snapshot, position, buttons, zoom,
        ) {
            // keep going to sync auto-pan timer
        } else if searcher::handle_searcher_pointer_move(self, cx, position, zoom) {
            // keep going to sync auto-pan timer
        } else if context_menu::handle_context_menu_pointer_move(self, cx, position, zoom) {
            // keep going to sync auto-pan timer
        } else {
            hover::update_hover_edge(self, cx, snapshot, position, zoom);
        }

        let snapshot = self.sync_view_state(cx.app);
        self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
    }
}
