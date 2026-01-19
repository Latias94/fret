use super::*;

pub(super) fn handle_context_menu_escape<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.context_menu.take().is_some() {
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    false
}

pub(super) fn handle_context_menu_key_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: fret_core::KeyCode,
) -> bool {
    let Some(mut menu) = canvas.interaction.context_menu.take() else {
        return false;
    };

    match key {
        fret_core::KeyCode::ArrowDown => {
            let n = menu.items.len();
            if n > 0 {
                let mut ix = (menu.active_item + 1) % n;
                for _ in 0..n {
                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                        break;
                    }
                    ix = (ix + 1) % n;
                }
                menu.active_item = ix;
            }
            menu.typeahead.clear();
            canvas.interaction.context_menu = Some(menu);
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::ArrowUp => {
            let n = menu.items.len();
            if n > 0 {
                let mut ix = if menu.active_item == 0 {
                    n - 1
                } else {
                    menu.active_item - 1
                };
                for _ in 0..n {
                    if menu.items.get(ix).is_some_and(|it| it.enabled) {
                        break;
                    }
                    ix = if ix == 0 { n - 1 } else { ix - 1 };
                }
                menu.active_item = ix;
            }
            menu.typeahead.clear();
            canvas.interaction.context_menu = Some(menu);
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Enter | fret_core::KeyCode::NumpadEnter => {
            let ix = menu.active_item.min(menu.items.len().saturating_sub(1));
            let item = menu.items.get(ix).cloned();
            let target = menu.target.clone();
            let invoked_at = menu.invoked_at;
            let candidates = menu.candidates.clone();

            if let Some(item) = item
                && item.enabled
            {
                canvas.activate_context_menu_item(cx, &target, invoked_at, item, &candidates);
            }

            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            return true;
        }
        fret_core::KeyCode::Backspace => {
            if !menu.typeahead.is_empty() {
                menu.typeahead.pop();
                canvas.interaction.context_menu = Some(menu);
                cx.stop_propagation();
                cx.request_redraw();
                cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
                return true;
            }
        }
        _ => {}
    }

    if let Some(ch) = fret_core::keycode_to_ascii_lowercase(key) {
        let try_find = |needle: &str| -> Option<usize> {
            if needle.is_empty() {
                return None;
            }
            menu.items.iter().position(|it| {
                it.enabled && it.label.as_ref().to_ascii_lowercase().starts_with(needle)
            })
        };

        menu.typeahead.push(ch);
        let mut needle = menu.typeahead.to_ascii_lowercase();
        let mut hit = try_find(&needle);
        if hit.is_none() {
            needle.clear();
            needle.push(ch);
            hit = try_find(&needle);
            if hit.is_some() {
                menu.typeahead.clear();
                menu.typeahead.push(ch);
            }
        }

        if let Some(ix) = hit {
            menu.active_item = ix.min(menu.items.len().saturating_sub(1));
        }

        canvas.interaction.context_menu = Some(menu);
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }

    canvas.interaction.context_menu = Some(menu);
    false
}

pub(super) fn handle_context_menu_pointer_down<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    let Some(menu) = canvas.interaction.context_menu.take() else {
        return false;
    };

    match button {
        MouseButton::Left => {
            if let Some(ix) = super::hit_context_menu_item(&canvas.style, &menu, position, zoom) {
                let item = menu.items.get(ix).cloned();
                let target = menu.target.clone();
                let invoked_at = menu.invoked_at;
                let candidates = menu.candidates.clone();
                if let Some(item) = item
                    && item.enabled
                {
                    canvas.activate_context_menu_item(cx, &target, invoked_at, item, &candidates);
                }
            } else {
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        MouseButton::Right => false,
        _ => {
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
    }
}

pub(super) fn handle_context_menu_pointer_move<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(menu) = canvas.interaction.context_menu.as_mut() else {
        return false;
    };

    let new_hover = super::hit_context_menu_item(&canvas.style, menu, position, zoom);
    if menu.hovered_item != new_hover {
        menu.hovered_item = new_hover;
        if let Some(ix) = new_hover {
            if menu.items.get(ix).is_some_and(|it| it.enabled) {
                menu.active_item = ix.min(menu.items.len().saturating_sub(1));
                menu.typeahead.clear();
            }
        }
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    }

    true
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_context_menu_item<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        target: &ContextMenuTarget,
        invoked_at: Point,
        item: NodeGraphContextMenuItem,
        menu_candidates: &[InsertNodeCandidate],
    ) {
        match (target, item.action) {
            (_, NodeGraphContextMenuAction::Command(command)) => {
                self.interaction.context_menu = None;
                if let ContextMenuTarget::Group(group_id) = target {
                    let group_id = *group_id;
                    self.update_view_state(cx.app, |s| {
                        s.selected_nodes.clear();
                        s.selected_edges.clear();
                        if !s.selected_groups.iter().any(|id| *id == group_id) {
                            s.selected_groups.clear();
                            s.selected_groups.push(group_id);
                        }
                    });
                }
                cx.dispatch_command(command);
            }
            (
                ContextMenuTarget::BackgroundInsertNodePicker { at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let outcome = if candidate.kind.0 == REROUTE_KIND {
                    Some(Ok(Self::build_reroute_create_ops(*at)))
                } else {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.plan_create_node(graph, &candidate, *at)
                        })
                        .ok()
                };

                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
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
                    Some(Err(msg)) => {
                        self.show_toast(cx.app, cx.window, DiagnosticSeverity::Info, msg)
                    }
                    None => {}
                }
            }
            (
                ContextMenuTarget::ConnectionInsertNodePicker { from, at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let mode = self.sync_view_state(cx.app).interaction.connection_mode;

                enum Outcome {
                    Apply(Vec<GraphOp>, Option<GraphNodeId>, Option<PortId>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let (outcome, toast) = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let insert_ops = if candidate.kind.0 == REROUTE_KIND {
                                Ok(Self::build_reroute_create_ops(*at))
                            } else {
                                presenter.plan_create_node(graph, &candidate, *at)
                            };

                            let insert_ops = match insert_ops {
                                Ok(ops) => ops,
                                Err(msg) => {
                                    return (Outcome::Reject(DiagnosticSeverity::Info, msg), None);
                                }
                            };

                            let planned = workflow::plan_wire_drop_insert(
                                presenter, graph, *from, mode, insert_ops,
                            );
                            let toast = planned.toast.clone();
                            (
                                Outcome::Apply(
                                    planned.ops,
                                    planned.created_node,
                                    planned.continue_from,
                                ),
                                toast,
                            )
                        })
                        .ok()
                        .unwrap_or((Outcome::Ignore, None))
                };

                match outcome {
                    Outcome::Apply(ops, created_node, continue_from) => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                            if let Some(node_id) = created_node {
                                self.update_view_state(cx.app, |s| {
                                    s.selected_edges.clear();
                                    s.selected_groups.clear();
                                    s.selected_nodes.clear();
                                    s.selected_nodes.push(node_id);
                                    s.draw_order.retain(|id| *id != node_id);
                                    s.draw_order.push(node_id);
                                });
                            }
                            if let Some((sev, msg)) = toast {
                                self.show_toast(cx.app, cx.window, sev, msg);
                            }

                            if let Some(port) = continue_from {
                                self.interaction.suspended_wire_drag = None;
                                self.start_sticky_wire_drag_from_port(cx, port, resume_pos);
                            } else {
                                self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                            }
                        } else {
                            self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                        }
                    }
                    Outcome::Reject(sev, msg) => {
                        self.show_toast(cx.app, cx.window, sev, msg);
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                    Outcome::Ignore => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                }
            }
            (
                ContextMenuTarget::Edge(edge_id),
                NodeGraphContextMenuAction::OpenInsertNodePicker,
            ) => {
                edge_insert::open_edge_insert_context_menu(self, cx, *edge_id, invoked_at);
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::InsertReroute) => {
                let at = self.reroute_pos_for_invoked_at(invoked_at);
                let kind = NodeKindKey::new(REROUTE_KIND);

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let plan = presenter.plan_split_edge(graph, *edge_id, &kind, at);
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
                        self.apply_ops(cx.app, cx.window, ops);
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
                    Some(Err(diags)) => {
                        let (sev, msg) =
                            Self::toast_from_diagnostics(&diags).unwrap_or_else(|| {
                                (
                                    DiagnosticSeverity::Error,
                                    Arc::<str>::from("failed to insert reroute"),
                                )
                            });
                        self.show_toast(cx.app, cx.window, sev, msg);
                    }
                    None => {}
                }
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::DeleteEdge) => {
                let remove_ops = {
                    let this = &*self;
                    this.graph
                        .read_ref(cx.app, |graph| {
                            graph
                                .edges
                                .get(edge_id)
                                .map(|edge| {
                                    vec![GraphOp::RemoveEdge {
                                        id: *edge_id,
                                        edge: edge.clone(),
                                    }]
                                })
                                .unwrap_or_default()
                        })
                        .ok()
                        .unwrap_or_default()
                };

                self.apply_ops(cx.app, cx.window, remove_ops);
                self.update_view_state(cx.app, |s| {
                    s.selected_edges.retain(|id| *id != *edge_id);
                });
            }
            (
                ContextMenuTarget::EdgeInsertNodePicker(edge_id),
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                edge_insert::insert_node_on_edge(self, cx, *edge_id, invoked_at, candidate);
            }
            (
                ContextMenuTarget::ConnectionConvertPicker { from, to, at },
                NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix),
            ) => {
                enum Outcome {
                    Apply(Vec<GraphOp>),
                    Reject(DiagnosticSeverity, Arc<str>),
                    Ignore,
                }

                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);

                let zoom = self.cached_zoom;
                let style = self.style.clone();

                let outcome = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            let template = match &candidate.template {
                                Some(t) => t,
                                None => {
                                    return Outcome::Reject(
                                        DiagnosticSeverity::Error,
                                        Arc::<str>::from(
                                            "conversion candidate is missing template",
                                        ),
                                    );
                                }
                            };

                            let plan = conversion::plan_insert_conversion(
                                presenter, graph, &style, zoom, *from, *to, *at, template,
                            );
                            match plan.decision {
                                ConnectDecision::Accept => Outcome::Apply(plan.ops),
                                ConnectDecision::Reject => {
                                    Self::toast_from_diagnostics(&plan.diagnostics)
                                        .map(|(sev, msg)| Outcome::Reject(sev, msg))
                                        .unwrap_or(Outcome::Ignore)
                                }
                            }
                        })
                        .ok()
                        .unwrap_or(Outcome::Ignore)
                };

                match outcome {
                    Outcome::Apply(ops) => {
                        let node_id = Self::first_added_node_id(&ops);
                        self.apply_ops(cx.app, cx.window, ops);
                        self.interaction.suspended_wire_drag = None;
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
                    Outcome::Reject(sev, msg) => {
                        self.show_toast(cx.app, cx.window, sev, msg);
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                    Outcome::Ignore => {
                        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
                        self.restore_suspended_wire_drag(cx, Some(*from), resume_pos);
                    }
                }
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::Custom(action_id)) => {
                let ops = {
                    let presenter = &mut *self.presenter;
                    self.graph
                        .read_ref(cx.app, |graph| {
                            presenter.on_edge_context_menu_action(graph, *edge_id, action_id)
                        })
                        .ok()
                        .flatten()
                        .unwrap_or_default()
                };

                if !ops.is_empty() {
                    self.apply_ops(cx.app, cx.window, ops);
                }
            }
            _ => {}
        }
    }
}
