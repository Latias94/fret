use crate::ui::canvas::widget::*;
impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn activate_context_menu_item<H: UiHost>(
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
