use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn handle_event<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        event: &Event,
        snapshot: &ViewSnapshot,
    ) {
        let zoom = snapshot.zoom;

        match event {
            Event::ClipboardText { token, text } => {
                let Some(pending) = self.interaction.pending_paste.take() else {
                    return;
                };
                if pending.token != *token {
                    self.interaction.pending_paste = Some(pending);
                    return;
                }
                self.apply_paste_text(cx.app, cx.window, text, pending.at);
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            Event::ClipboardTextUnavailable { token } => {
                if let Some(pending) = &self.interaction.pending_paste
                    && pending.token == *token
                {
                    self.interaction.pending_paste = None;
                    self.show_toast(
                        cx.app,
                        cx.window,
                        DiagnosticSeverity::Info,
                        "clipboard text unavailable",
                    );
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::WindowFocusChanged(false) => {
                if self.interaction.searcher.is_some() || self.interaction.context_menu.is_some() {
                    return;
                }

                cancel::handle_escape_cancel(self, cx);
                self.interaction.pan_activation_key_held = false;
                self.interaction.multi_selection_active = false;
                return;
            }
            Event::PointerCancel(_) => {
                cancel::cancel_active_gestures(self, cx);
                return;
            }
            Event::InternalDrag(e) => {
                if insert_node_drag::handle_internal_drag_event(self, cx, &snapshot, e, zoom) {
                    return;
                }
            }
            Event::Timer { token } => {
                if self
                    .interaction
                    .toast
                    .as_ref()
                    .is_some_and(|t| t.timer == *token)
                {
                    self.interaction.toast = None;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    return;
                }

                if self
                    .interaction
                    .pan_inertia
                    .as_ref()
                    .is_some_and(|i| i.timer == *token)
                {
                    let tuning = snapshot.interaction.pan_inertia.clone();
                    let zoom = snapshot.zoom;
                    let before = snapshot.pan;

                    let Some(mut inertia) = self.interaction.pan_inertia.take() else {
                        return;
                    };
                    let timer = inertia.timer;
                    let mut end_move = false;

                    if !tuning.enabled
                        || !self.pan_inertia_should_tick()
                        || !zoom.is_finite()
                        || zoom <= 0.0
                        || !tuning.decay_per_s.is_finite()
                        || tuning.decay_per_s <= 0.0
                    {
                        cx.app.push_effect(Effect::CancelTimer { token: timer });
                        end_move = true;
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        if end_move {
                            let snap = self.sync_view_state(cx.app);
                            self.emit_move_end(
                                &snap,
                                ViewportMoveKind::PanInertia,
                                ViewportMoveEndOutcome::Ended,
                            );
                        }
                        return;
                    }

                    let now = Instant::now();
                    let dt = (now - inertia.last_tick_at).as_secs_f32().clamp(0.0, 0.2);
                    inertia.last_tick_at = now;

                    if dt > 0.0 {
                        let dx = inertia.velocity.x * dt;
                        let dy = inertia.velocity.y * dt;
                        self.update_view_state(cx.app, |s| {
                            s.pan.x += dx;
                            s.pan.y += dy;
                        });
                    }

                    let after = self.sync_view_state(cx.app).pan;
                    let moved_x = after.x - before.x;
                    let moved_y = after.y - before.y;
                    let moved = (moved_x * moved_x + moved_y * moved_y).sqrt();

                    let decay = (-tuning.decay_per_s * dt).exp();
                    inertia.velocity.x *= decay;
                    inertia.velocity.y *= decay;

                    let speed_screen = (inertia.velocity.x * inertia.velocity.x
                        + inertia.velocity.y * inertia.velocity.y)
                        .sqrt()
                        * zoom;
                    let min_speed = tuning.min_speed.max(0.0);

                    if moved <= 1.0e-6
                        || !speed_screen.is_finite()
                        || speed_screen <= min_speed
                        || !inertia.velocity.x.is_finite()
                        || !inertia.velocity.y.is_finite()
                    {
                        cx.app.push_effect(Effect::CancelTimer { token: timer });
                        end_move = true;
                    } else {
                        self.interaction.pan_inertia = Some(inertia);
                    }

                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                    if end_move {
                        let snap = self.sync_view_state(cx.app);
                        self.emit_move_end(
                            &snap,
                            ViewportMoveKind::PanInertia,
                            ViewportMoveEndOutcome::Ended,
                        );
                    }
                    return;
                }

                if self.interaction.auto_pan_timer == Some(*token) {
                    if !self.auto_pan_should_tick(&snapshot, cx.bounds) {
                        self.stop_auto_pan_timer(cx.app);
                        return;
                    }

                    let pos = self.interaction.last_pos.unwrap_or_default();
                    let mods = self.interaction.last_modifiers;
                    let zoom = snapshot.zoom;

                    if self.interaction.wire_drag.is_some() {
                        let _ =
                            wire_drag::handle_wire_drag_move(self, cx, &snapshot, pos, mods, zoom);
                    } else if self.interaction.node_drag.is_some() {
                        let _ =
                            node_drag::handle_node_drag_move(self, cx, &snapshot, pos, mods, zoom);
                    } else if self.interaction.group_drag.is_some() {
                        let _ = group_drag::handle_group_drag_move(
                            self, cx, &snapshot, pos, mods, zoom,
                        );
                    } else if self.interaction.group_resize.is_some() {
                        let _ = group_resize::handle_group_resize_move(
                            self, cx, &snapshot, pos, mods, zoom,
                        );
                    }

                    let snapshot = self.sync_view_state(cx.app);
                    self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }

                if self
                    .interaction
                    .viewport_move_debounce
                    .as_ref()
                    .is_some_and(|s| s.timer == *token)
                {
                    let Some(state) = self.interaction.viewport_move_debounce.take() else {
                        return;
                    };
                    let snapshot = self.sync_view_state(cx.app);
                    self.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::KeyDown { key, modifiers, .. } => {
                if cx.input_ctx.focus_is_text_input {
                    return;
                }

                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

                if *key == fret_core::KeyCode::Escape {
                    if searcher::handle_searcher_escape(self, cx)
                        || context_menu::handle_context_menu_escape(self, cx)
                    {
                        return;
                    }
                    cancel::handle_escape_cancel(self, cx);
                    return;
                }

                if searcher::handle_searcher_key_down(self, cx, *key, *modifiers)
                    || context_menu::handle_context_menu_key_down(self, cx, *key)
                {
                    return;
                }

                if modifiers.ctrl || modifiers.meta {
                    if !snapshot.interaction.disable_keyboard_a11y
                        && *key == fret_core::KeyCode::Tab
                    {
                        let cmd = if modifiers.shift {
                            CMD_NODE_GRAPH_FOCUS_PREV_EDGE
                        } else {
                            CMD_NODE_GRAPH_FOCUS_NEXT_EDGE
                        };
                        cx.dispatch_command(CommandId::from(cmd));
                        cx.stop_propagation();
                        return;
                    }

                    match *key {
                        fret_core::KeyCode::KeyA => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_SELECT_ALL));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyZ => {
                            let cmd = if modifiers.shift {
                                CMD_NODE_GRAPH_REDO
                            } else {
                                CMD_NODE_GRAPH_UNDO
                            };
                            cx.dispatch_command(CommandId::from(cmd));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyY => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_REDO));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyC => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_COPY));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyX => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_CUT));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyV => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_PASTE));
                            cx.stop_propagation();
                            return;
                        }
                        fret_core::KeyCode::KeyD => {
                            cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DUPLICATE));
                            cx.stop_propagation();
                            return;
                        }
                        _ => {}
                    }
                }

                if !snapshot.interaction.disable_keyboard_a11y
                    && *key == fret_core::KeyCode::Tab
                    && !modifiers.ctrl
                    && !modifiers.meta
                    && !modifiers.alt
                    && !modifiers.alt_gr
                {
                    if self.interaction.searcher.is_some()
                        || self.interaction.context_menu.is_some()
                    {
                        return;
                    }

                    let cmd = if modifiers.shift {
                        CMD_NODE_GRAPH_FOCUS_PREV
                    } else {
                        CMD_NODE_GRAPH_FOCUS_NEXT
                    };
                    cx.dispatch_command(CommandId::from(cmd));
                    cx.stop_propagation();
                    return;
                }

                if !modifiers.ctrl && !modifiers.meta && !modifiers.alt && !modifiers.alt_gr {
                    if snapshot.interaction.space_to_pan
                        && self.interaction.searcher.is_none()
                        && self.interaction.context_menu.is_none()
                    {
                        if let Some(crate::io::NodeGraphKeyCode(key_code)) =
                            snapshot.interaction.pan_activation_key_code
                        {
                            if *key == key_code && !self.interaction.pan_activation_key_held {
                                self.interaction.pan_activation_key_held = true;
                                cx.request_redraw();
                                cx.invalidate_self(Invalidation::Paint);
                                cx.stop_propagation();
                                return;
                            }
                        }
                    }
                }

                if matches!(
                    key,
                    fret_core::KeyCode::ArrowLeft
                        | fret_core::KeyCode::ArrowRight
                        | fret_core::KeyCode::ArrowUp
                        | fret_core::KeyCode::ArrowDown
                ) && !modifiers.ctrl
                    && !modifiers.meta
                    && !modifiers.alt
                    && !modifiers.alt_gr
                {
                    if snapshot.interaction.disable_keyboard_a11y {
                        return;
                    }

                    if snapshot.selected_nodes.is_empty() && snapshot.selected_groups.is_empty() {
                        return;
                    }

                    let cmd = match (*key, modifiers.shift) {
                        (fret_core::KeyCode::ArrowLeft, false) => CMD_NODE_GRAPH_NUDGE_LEFT,
                        (fret_core::KeyCode::ArrowRight, false) => CMD_NODE_GRAPH_NUDGE_RIGHT,
                        (fret_core::KeyCode::ArrowUp, false) => CMD_NODE_GRAPH_NUDGE_UP,
                        (fret_core::KeyCode::ArrowDown, false) => CMD_NODE_GRAPH_NUDGE_DOWN,
                        (fret_core::KeyCode::ArrowLeft, true) => CMD_NODE_GRAPH_NUDGE_LEFT_FAST,
                        (fret_core::KeyCode::ArrowRight, true) => CMD_NODE_GRAPH_NUDGE_RIGHT_FAST,
                        (fret_core::KeyCode::ArrowUp, true) => CMD_NODE_GRAPH_NUDGE_UP_FAST,
                        (fret_core::KeyCode::ArrowDown, true) => CMD_NODE_GRAPH_NUDGE_DOWN_FAST,
                        _ => return,
                    };
                    cx.dispatch_command(CommandId::from(cmd));
                    cx.stop_propagation();
                    return;
                }

                if !snapshot.interaction.delete_key.matches(*key) {
                    return;
                }

                cx.dispatch_command(CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION));
                cx.stop_propagation();
                return;
            }
            Event::KeyUp { key, .. } => {
                let Some(crate::io::NodeGraphKeyCode(key_code)) =
                    snapshot.interaction.pan_activation_key_code
                else {
                    return;
                };
                if *key == key_code && self.interaction.pan_activation_key_held {
                    self.interaction.pan_activation_key_held = false;
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
                return;
            }
            Event::Pointer(fret_core::PointerEvent::Down {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                self.interaction.last_pos = Some(*position);
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);
                self.interaction.last_canvas_pos = Some(CanvasPoint {
                    x: position.x.0,
                    y: position.y.0,
                });

                if searcher::handle_searcher_pointer_down(self, cx, *position, *button, zoom) {
                    return;
                }

                if *button == MouseButton::Left {
                    if let Some(command) = self.close_command.clone() {
                        let rect = Self::close_button_rect(snapshot.pan, zoom);
                        if Self::rect_contains(rect, *position) {
                            cx.dispatch_command(command);
                            cx.stop_propagation();
                            return;
                        }
                    }
                }

                if *button == MouseButton::Left
                    && *click_count == 2
                    && snapshot.interaction.zoom_on_double_click
                    && self.interaction.searcher.is_none()
                    && self.interaction.context_menu.is_none()
                {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let is_background = self
                        .graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_ports: Vec<PortId> = Vec::new();
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();

                            if self
                                .hit_port(
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_ports,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            if self
                                .hit_edge_focus_anchor(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            if geom.nodes.values().any(|ng| ng.rect.contains(*position)) {
                                return false;
                            }
                            if self
                                .hit_edge(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return false;
                            }
                            !graph.groups.iter().any(|(group_id, group)| {
                                let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                                group_resize::group_rect_to_px(rect0).contains(*position)
                            })
                        })
                        .unwrap_or(false);

                    if is_background {
                        if let Some(state) = self.interaction.viewport_move_debounce.take() {
                            cx.app
                                .push_effect(Effect::CancelTimer { token: state.timer });
                            self.emit_move_end(
                                &snapshot,
                                state.kind,
                                ViewportMoveEndOutcome::Ended,
                            );
                        }

                        self.emit_move_start(&snapshot, ViewportMoveKind::ZoomDoubleClick);
                        let factor = if modifiers.shift { 0.5 } else { 2.0 };
                        self.zoom_about_pointer_factor(*position, factor);
                        let pan = self.cached_pan;
                        let zoom = self.cached_zoom;
                        self.update_view_state(cx.app, |s| {
                            s.pan = pan;
                            s.zoom = zoom;
                        });
                        let snap = self.sync_view_state(cx.app);
                        self.emit_move_end(
                            &snap,
                            ViewportMoveKind::ZoomDoubleClick,
                            ViewportMoveEndOutcome::Ended,
                        );
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if *button == MouseButton::Left
                    && *click_count == 2
                    && (modifiers.alt || modifiers.alt_gr)
                    && self.interaction.searcher.is_none()
                    && self.interaction.context_menu.is_none()
                {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let hit: Option<EdgeId> = self
                        .graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_ports: Vec<PortId> = Vec::new();
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();

                            if self
                                .hit_port(
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_ports,
                                )
                                .is_some()
                            {
                                return None;
                            }
                            if self
                                .hit_edge_focus_anchor(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return None;
                            }
                            if geom.nodes.values().any(|ng| ng.rect.contains(*position)) {
                                return None;
                            }
                            if graph.groups.iter().any(|(group_id, group)| {
                                let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                                group_resize::group_rect_to_px(rect0).contains(*position)
                            }) {
                                return None;
                            }
                            self.hit_edge(
                                graph,
                                &snapshot,
                                geom.as_ref(),
                                index.as_ref(),
                                *position,
                                zoom,
                                &mut scratch_edges,
                            )
                        })
                        .ok()
                        .flatten();

                    if let Some(edge_id) = hit {
                        self.update_view_state(cx.app, |s| {
                            s.selected_nodes.clear();
                            s.selected_groups.clear();
                            if !s.selected_edges.iter().any(|id| *id == edge_id) {
                                s.selected_edges.clear();
                                s.selected_edges.push(edge_id);
                            }
                        });
                        self.open_edge_insert_node_picker(cx.app, cx.window, edge_id, *position);
                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if *button == MouseButton::Left
                    && *click_count == 2
                    && snapshot.interaction.reroute_on_edge_double_click
                    && self.interaction.searcher.is_none()
                    && self.interaction.context_menu.is_none()
                {
                    let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
                    let hit: Option<EdgeId> = self
                        .graph
                        .read_ref(cx.app, |graph| {
                            let mut scratch_ports: Vec<PortId> = Vec::new();
                            let mut scratch_edges: Vec<EdgeId> = Vec::new();

                            if self
                                .hit_port(
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_ports,
                                )
                                .is_some()
                            {
                                return None;
                            }
                            if self
                                .hit_edge_focus_anchor(
                                    graph,
                                    &snapshot,
                                    geom.as_ref(),
                                    index.as_ref(),
                                    *position,
                                    zoom,
                                    &mut scratch_edges,
                                )
                                .is_some()
                            {
                                return None;
                            }
                            if geom.nodes.values().any(|ng| ng.rect.contains(*position)) {
                                return None;
                            }
                            if graph.groups.iter().any(|(group_id, group)| {
                                let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                                group_resize::group_rect_to_px(rect0).contains(*position)
                            }) {
                                return None;
                            }
                            self.hit_edge(
                                graph,
                                &snapshot,
                                geom.as_ref(),
                                index.as_ref(),
                                *position,
                                zoom,
                                &mut scratch_edges,
                            )
                        })
                        .ok()
                        .flatten();

                    if let Some(edge_id) = hit {
                        let invoked_at = *position;
                        let at = self.reroute_pos_for_invoked_at(invoked_at);

                        let outcome = {
                            let presenter = &mut *self.presenter;
                            self.graph
                                .read_ref(cx.app, |graph| {
                                    let plan = presenter.plan_split_edge(
                                        graph,
                                        edge_id,
                                        &NodeKindKey::new(REROUTE_KIND),
                                        at,
                                    );
                                    match plan.decision {
                                        ConnectDecision::Accept => Ok(plan.ops),
                                        ConnectDecision::Reject => Err(plan.diagnostics),
                                    }
                                })
                                .ok()
                        };

                        match outcome {
                            Some(Ok(ops)) => {
                                let node_id = Self::first_added_node_id(&ops);
                                if self.commit_ops(cx.app, cx.window, Some("Insert Reroute"), ops) {
                                    if let Some(node_id) = node_id {
                                        self.update_view_state(cx.app, |s| {
                                            s.selected_edges.clear();
                                            s.selected_groups.clear();
                                            s.selected_nodes.clear();
                                            s.selected_nodes.push(node_id);
                                            s.draw_order.retain(|id| *id != node_id);
                                            s.draw_order.push(node_id);
                                        });
                                    }
                                }
                            }
                            Some(Err(diags)) => {
                                if let Some((sev, msg)) = Self::toast_from_diagnostics(&diags) {
                                    self.show_toast(cx.app, cx.window, sev, msg);
                                }
                            }
                            None => {}
                        }

                        cx.stop_propagation();
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if self.interaction.context_menu.is_some()
                    && context_menu::handle_context_menu_pointer_down(
                        self, cx, *position, *button, zoom,
                    )
                {
                    return;
                }

                if *button == MouseButton::Right {
                    cancel::cancel_active_gestures(self, cx);
                    if snapshot.interaction.pan_on_drag.right {
                        self.interaction.pending_right_click =
                            Some(super::super::state::PendingRightClick {
                                start_pos: *position,
                            });
                        cx.capture_pointer(cx.node);
                        cx.request_redraw();
                        cx.invalidate_self(Invalidation::Paint);
                        return;
                    }
                }

                if sticky_wire::handle_sticky_wire_pointer_down(
                    self, cx, &snapshot, *position, *button, zoom,
                ) {
                    return;
                }

                if *button == MouseButton::Left
                    && snapshot.interaction.space_to_pan
                    && self.interaction.pan_activation_key_held
                    && !(modifiers.ctrl || modifiers.meta || modifiers.alt || modifiers.alt_gr)
                {
                    let _ = pan_zoom::begin_panning(
                        self,
                        cx,
                        &snapshot,
                        *position,
                        fret_core::MouseButton::Left,
                    );
                    return;
                }

                if *button == MouseButton::Middle && snapshot.interaction.pan_on_drag.middle {
                    let _ = pan_zoom::begin_panning(
                        self,
                        cx,
                        &snapshot,
                        *position,
                        fret_core::MouseButton::Middle,
                    );
                    return;
                }

                if *button == MouseButton::Right
                    && right_click::handle_right_click_pointer_down(
                        self, cx, &snapshot, *position, zoom,
                    )
                {
                    return;
                }

                if *button != MouseButton::Left {
                    return;
                }

                let _ = left_click::handle_left_click_pointer_down(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                );
            }
            Event::Pointer(fret_core::PointerEvent::Move {
                position,
                buttons,
                modifiers,
                ..
            }) => {
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

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
                            *position,
                            button,
                            1,
                            *modifiers,
                            snapshot.zoom,
                        );
                        return;
                    }
                }

                if snapshot.interaction.pan_on_drag.right
                    && buttons.right
                    && self.interaction.panning_button.is_none()
                    && let Some(pending) = self.interaction.pending_right_click
                {
                    let click_distance = snapshot.interaction.pane_click_distance.max(0.0);
                    let threshold = canvas_units_from_screen_px(click_distance, zoom);
                    let dx = position.x.0 - pending.start_pos.x.0;
                    let dy = position.y.0 - pending.start_pos.y.0;
                    if click_distance == 0.0 || (dx * dx + dy * dy) > threshold * threshold {
                        self.interaction.pending_right_click = None;
                        let _ = pan_zoom::begin_panning(
                            self,
                            cx,
                            &snapshot,
                            *position,
                            fret_core::MouseButton::Right,
                        );
                        return;
                    }
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
                        *position,
                        fret_core::MouseButton::Left,
                        1,
                        *modifiers,
                        snapshot.zoom,
                    );
                    return;
                }

                if self.interaction.last_pos.is_none() {
                    self.interaction.last_pos = Some(*position);
                    self.interaction.last_modifiers = *modifiers;
                    self.interaction.last_canvas_pos = Some(CanvasPoint {
                        x: position.x.0,
                        y: position.y.0,
                    });
                    return;
                }
                self.interaction.last_pos = Some(*position);
                self.interaction.last_modifiers = *modifiers;
                self.interaction.last_canvas_pos = Some(CanvasPoint {
                    x: position.x.0,
                    y: position.y.0,
                });

                cursor::update_cursors(self, cx, &snapshot, *position, zoom);

                if pan_zoom::handle_panning_move(self, cx, &snapshot, *position) {
                    // keep going to sync auto-pan timer
                } else if marquee::handle_marquee_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_group_drag::handle_pending_group_drag_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if group_drag::handle_group_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_group_resize::handle_pending_group_resize_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if group_resize::handle_group_resize_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_drag::handle_pending_node_drag_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_resize::handle_pending_node_resize_move(
                    self, cx, &snapshot, *position, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if pending_wire_drag::handle_pending_wire_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if edge_insert_drag::handle_pending_edge_insert_drag_move(
                    self, cx, &snapshot, *position,
                ) {
                    // keep going to sync auto-pan timer
                } else if node_resize::handle_node_resize_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if node_drag::handle_node_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if wire_drag::handle_wire_drag_move(
                    self, cx, &snapshot, *position, *modifiers, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if edge_insert_drag::handle_edge_insert_drag_move(self, cx, *position) {
                    // keep going to sync auto-pan timer
                } else if edge_drag::handle_edge_drag_move(self, cx, &snapshot, *position, zoom) {
                    // keep going to sync auto-pan timer
                } else if insert_node_drag::handle_pending_insert_node_drag_move(
                    self, cx, &snapshot, *position, *buttons, zoom,
                ) {
                    // keep going to sync auto-pan timer
                } else if searcher::handle_searcher_pointer_move(self, cx, *position, zoom) {
                    // keep going to sync auto-pan timer
                } else if context_menu::handle_context_menu_pointer_move(self, cx, *position, zoom)
                {
                    // keep going to sync auto-pan timer
                } else {
                    hover::update_hover_edge(self, cx, &snapshot, *position, zoom);
                }

                let snapshot = self.sync_view_state(cx.app);
                self.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
            }
            Event::Pointer(fret_core::PointerEvent::Up {
                position,
                button,
                modifiers,
                click_count,
                ..
            }) => {
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);

                if *button == MouseButton::Right
                    && snapshot.interaction.pan_on_drag.right
                    && let Some(pending) = self.interaction.pending_right_click.take()
                {
                    let click_distance = snapshot.interaction.pane_click_distance.max(0.0);
                    let threshold = canvas_units_from_screen_px(click_distance, zoom);
                    let dx = position.x.0 - pending.start_pos.x.0;
                    let dy = position.y.0 - pending.start_pos.y.0;
                    let is_click =
                        click_distance == 0.0 || (dx * dx + dy * dy) <= threshold * threshold;

                    cx.release_pointer_capture();
                    if is_click {
                        right_click::handle_right_click_pointer_down(
                            self, cx, &snapshot, *position, zoom,
                        );
                    }
                    return;
                }

                if *button == MouseButton::Left
                    && searcher::handle_searcher_pointer_up(self, cx, *position, *button, zoom)
                {
                    return;
                }
                if pointer_up::handle_pointer_up(
                    self,
                    cx,
                    &snapshot,
                    *position,
                    *button,
                    *click_count,
                    *modifiers,
                    zoom,
                ) {
                    return;
                }
            }
            Event::Pointer(fret_core::PointerEvent::Wheel {
                position,
                delta,
                modifiers,
                ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                self.interaction.last_modifiers = *modifiers;
                self.interaction.multi_selection_active = snapshot
                    .interaction
                    .multi_selection_key
                    .is_pressed(*modifiers);
                if searcher::handle_searcher_wheel(self, cx, *delta, *modifiers, zoom) {
                    return;
                }

                let zoom_active = snapshot
                    .interaction
                    .zoom_activation_key
                    .is_pressed(*modifiers);
                if snapshot.interaction.zoom_on_scroll && zoom_active {
                    self.bump_viewport_move_debounce(
                        cx.app,
                        cx.window,
                        &snapshot,
                        ViewportMoveKind::ZoomWheel,
                    );
                    let speed = snapshot.interaction.zoom_on_scroll_speed.max(0.0);
                    let delta_screen_y = delta.y.0 * zoom;
                    let factor = fret_canvas::view::wheel_zoom_factor(
                        delta_screen_y,
                        fret_canvas::view::DEFAULT_WHEEL_ZOOM_BASE,
                        fret_canvas::view::DEFAULT_WHEEL_ZOOM_STEP,
                        speed,
                    )
                    .unwrap_or(1.0);
                    self.zoom_about_pointer_factor(*position, factor);
                    let pan = self.cached_pan;
                    let zoom = self.cached_zoom;
                    self.update_view_state(cx.app, |s| {
                        s.pan = pan;
                        s.zoom = zoom;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                } else if snapshot.interaction.pan_on_scroll
                    || (snapshot.interaction.space_to_pan
                        && self.interaction.pan_activation_key_held)
                {
                    self.bump_viewport_move_debounce(
                        cx.app,
                        cx.window,
                        &snapshot,
                        ViewportMoveKind::PanScroll,
                    );
                    let mode = snapshot.interaction.pan_on_scroll_mode;
                    let speed = snapshot.interaction.pan_on_scroll_speed.max(0.0);
                    let dy_for_shift = delta.y.0;

                    let mut dx = delta.x.0;
                    let mut dy = delta.y.0;
                    match mode {
                        crate::io::NodeGraphPanOnScrollMode::Free => {}
                        crate::io::NodeGraphPanOnScrollMode::Horizontal => {
                            dy = 0.0;
                        }
                        crate::io::NodeGraphPanOnScrollMode::Vertical => {
                            dx = 0.0;
                        }
                    }

                    if cx.input_ctx.platform != fret_runtime::Platform::Macos
                        && modifiers.shift
                        && !matches!(mode, crate::io::NodeGraphPanOnScrollMode::Vertical)
                    {
                        dx = dy_for_shift;
                        dy = 0.0;
                    }
                    self.update_view_state(cx.app, |s| {
                        s.pan.x += dx * speed;
                        s.pan.y += dy * speed;
                    });
                    cx.request_redraw();
                    cx.invalidate_self(Invalidation::Paint);
                }
            }
            Event::Pointer(fret_core::PointerEvent::PinchGesture {
                position, delta, ..
            }) => {
                if self.interaction.pan_inertia.is_some() {
                    self.stop_pan_inertia_timer(cx.app);
                    self.emit_move_end(
                        &snapshot,
                        ViewportMoveKind::PanInertia,
                        ViewportMoveEndOutcome::Ended,
                    );
                }
                if !snapshot.interaction.zoom_on_pinch {
                    return;
                }
                if !delta.is_finite() {
                    return;
                }

                self.bump_viewport_move_debounce(
                    cx.app,
                    cx.window,
                    &snapshot,
                    ViewportMoveKind::ZoomPinch,
                );

                let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
                let delta = (*delta).clamp(-0.95, 10.0);
                let factor = (1.0 + delta * speed).max(0.01);
                self.zoom_about_pointer_factor(*position, factor);
                let pan = self.cached_pan;
                let zoom = self.cached_zoom;
                self.update_view_state(cx.app, |s| {
                    s.pan = pan;
                    s.zoom = zoom;
                });
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
            }
            _ => {}
        }
    }
}
