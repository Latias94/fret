use super::connection_execution::ConnectionInsertMenuPlan;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn activate_connection_insert_picker_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        match action {
            NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
                let mode = self.sync_view_state(cx.app).interaction.connection_mode;
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return true;
                };
                self.record_recent_kind(&candidate.kind);
                let plan =
                    self.plan_connection_insert_menu_candidate(cx.app, from, at, mode, &candidate);
                self.apply_connection_insert_menu_plan(cx, from, invoked_at, plan);
                true
            }
            _ => false,
        }
    }

    pub(super) fn plan_connection_insert_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        let insert_ops =
            Self::plan_insert_candidate_ops_with_graph(presenter, graph, candidate, at);
        let insert_ops = match insert_ops {
            Ok(ops) => ops,
            Err(msg) => {
                return ConnectionInsertMenuPlan::Reject(DiagnosticSeverity::Info, msg);
            }
        };
        ConnectionInsertMenuPlan::Apply(workflow::plan_wire_drop_insert(
            presenter, graph, from, mode, insert_ops,
        ))
    }

    pub(super) fn plan_connection_insert_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        at: CanvasPoint,
        mode: NodeGraphConnectionMode,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionInsertMenuPlan {
        let presenter = &mut *self.presenter;
        self.graph
            .read_ref(host, |graph| {
                Self::plan_connection_insert_menu_candidate_with_graph(
                    presenter, graph, from, at, mode, candidate,
                )
            })
            .ok()
            .unwrap_or(ConnectionInsertMenuPlan::Ignore)
    }

    pub(super) fn apply_connection_insert_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionInsertMenuPlan,
    ) {
        match plan {
            ConnectionInsertMenuPlan::Apply(planned) => {
                let workflow::WireDropInsertPlan {
                    ops,
                    created_node,
                    continue_from,
                    toast,
                } = planned;
                if self.commit_ops(cx.app, cx.window, Some("Insert Node"), ops) {
                    self.select_inserted_node(cx.app, created_node);
                    if let Some((severity, message)) = toast {
                        self.show_toast(cx.app, cx.window, severity, message);
                    }
                    self.resume_connection_insert_wire_drag(
                        cx,
                        fallback_from,
                        invoked_at,
                        continue_from,
                    );
                } else {
                    self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
                }
            }
            ConnectionInsertMenuPlan::Reject(severity, message) => {
                self.show_toast(cx.app, cx.window, severity, message);
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
            ConnectionInsertMenuPlan::Ignore => {
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
        }
    }

    fn resume_connection_insert_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        continue_from: Option<PortId>,
    ) {
        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
        if let Some(port) = continue_from {
            self.interaction.suspended_wire_drag = None;
            self.start_sticky_wire_drag_from_port(cx, port, resume_pos);
        } else {
            self.restore_suspended_wire_drag(cx, Some(fallback_from), resume_pos);
        }
    }

    pub(super) fn restore_connection_menu_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
        self.restore_suspended_wire_drag(cx, Some(fallback_from), resume_pos);
    }
}
