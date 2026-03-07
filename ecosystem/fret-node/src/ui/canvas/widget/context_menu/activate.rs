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
                let outcome = self.plan_canvas_insert_candidate_ops(cx.app, &candidate, *at);
                match outcome {
                    Some(Ok(ops)) => {
                        let node_id = Self::first_added_node_id(&ops);
                        if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                            self.select_inserted_node(cx.app, node_id);
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
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);
                let plan = self
                    .plan_connection_insert_menu_candidate(cx.app, *from, *at, mode, &candidate);
                self.apply_connection_insert_menu_plan(cx, *from, invoked_at, plan);
            }
            (
                ContextMenuTarget::Edge(edge_id),
                NodeGraphContextMenuAction::OpenInsertNodePicker,
            ) => {
                edge_insert::open_edge_insert_context_menu(self, cx, *edge_id, invoked_at);
            }
            (ContextMenuTarget::Edge(edge_id), NodeGraphContextMenuAction::InsertReroute) => {
                let outcome = self.plan_canvas_split_edge_reroute(cx.app, *edge_id, invoked_at);
                match outcome {
                    Some(Ok(ops)) => {
                        self.apply_split_edge_reroute_ops(cx.app, cx.window, None, ops);
                    }
                    Some(Err(diags)) => {
                        let (sev, msg) = Self::split_edge_reroute_rejection_toast(&diags);
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
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return;
                };
                self.record_recent_kind(&candidate.kind);
                let plan = self
                    .plan_connection_conversion_menu_candidate(cx.app, *from, *to, *at, &candidate);
                self.apply_connection_conversion_menu_plan(cx, *from, invoked_at, plan);
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
