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
            && click_count == 2
            && snapshot.interaction.zoom_on_double_click
            && self.interaction.searcher.is_none()
            && self.interaction.context_menu.is_none()
        {
            let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
            let is_background = self
                .graph
                .read_ref(cx.app, |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

                    if self.hit_port(&mut ctx, position).is_some() {
                        return false;
                    }
                    if self
                        .hit_edge_focus_anchor(graph, &snapshot, &mut ctx, position)
                        .is_some()
                    {
                        return false;
                    }
                    if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                        return false;
                    }
                    if self
                        .hit_edge(graph, &snapshot, &mut ctx, position)
                        .is_some()
                    {
                        return false;
                    }
                    !graph.groups.iter().any(|(group_id, group)| {
                        let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                        group_resize::group_rect_to_px(rect0).contains(position)
                    })
                })
                .unwrap_or(false);

            if is_background {
                if let Some(state) = self.interaction.viewport_move_debounce.take() {
                    cx.app
                        .push_effect(Effect::CancelTimer { token: state.timer });
                    self.emit_move_end(&snapshot, state.kind, ViewportMoveEndOutcome::Ended);
                }

                self.emit_move_start(&snapshot, ViewportMoveKind::ZoomDoubleClick);
                let factor = if modifiers.shift { 0.5 } else { 2.0 };
                self.zoom_about_pointer_factor(position, factor);
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

        if button == MouseButton::Left
            && click_count == 2
            && (modifiers.alt || modifiers.alt_gr)
            && self.interaction.searcher.is_none()
            && self.interaction.context_menu.is_none()
        {
            let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
            let hit: Option<EdgeId> = self
                .graph
                .read_ref(cx.app, |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

                    if self.hit_port(&mut ctx, position).is_some() {
                        return None;
                    }
                    if self
                        .hit_edge_focus_anchor(graph, &snapshot, &mut ctx, position)
                        .is_some()
                    {
                        return None;
                    }
                    if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                        return None;
                    }
                    if graph.groups.iter().any(|(group_id, group)| {
                        let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                        group_resize::group_rect_to_px(rect0).contains(position)
                    }) {
                        return None;
                    }
                    self.hit_edge(graph, &snapshot, &mut ctx, position)
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
                self.open_edge_insert_node_picker(cx.app, cx.window, edge_id, position);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(Invalidation::Paint);
                return;
            }
        }

        if button == MouseButton::Left
            && click_count == 2
            && snapshot.interaction.reroute_on_edge_double_click
            && self.interaction.searcher.is_none()
            && self.interaction.context_menu.is_none()
        {
            let (geom, index) = self.canvas_derived(&*cx.app, &snapshot);
            let hit: Option<EdgeId> = self
                .graph
                .read_ref(cx.app, |graph| {
                    let mut scratch = HitTestScratch::default();
                    let mut ctx =
                        HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

                    if self.hit_port(&mut ctx, position).is_some() {
                        return None;
                    }
                    if self
                        .hit_edge_focus_anchor(graph, &snapshot, &mut ctx, position)
                        .is_some()
                    {
                        return None;
                    }
                    if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                        return None;
                    }
                    if graph.groups.iter().any(|(group_id, group)| {
                        let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                        group_resize::group_rect_to_px(rect0).contains(position)
                    }) {
                        return None;
                    }
                    self.hit_edge(graph, &snapshot, &mut ctx, position)
                })
                .ok()
                .flatten();

            if let Some(edge_id) = hit {
                let invoked_at = position;
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
            && context_menu::handle_context_menu_pointer_down(self, cx, position, button, zoom)
        {
            return;
        }

        if button == MouseButton::Right {
            cancel::cancel_active_gestures(self, cx);
            if snapshot.interaction.pan_on_drag.right {
                self.interaction.pending_right_click =
                    Some(super::super::state::PendingRightClick {
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
