use super::connection_execution::ConnectionConversionMenuPlan;
use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_connection_conversion_menu_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        style: &NodeGraphStyle,
        zoom: f32,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        let template = match &candidate.template {
            Some(template) => template,
            None => {
                return ConnectionConversionMenuPlan::Reject(
                    DiagnosticSeverity::Error,
                    Arc::<str>::from("conversion candidate is missing template"),
                );
            }
        };
        let plan = conversion::plan_insert_conversion(
            presenter, graph, style, zoom, from, to, at, template,
        );
        match plan.decision {
            ConnectDecision::Accept => ConnectionConversionMenuPlan::Apply(plan.ops),
            ConnectDecision::Reject => Self::toast_from_diagnostics(&plan.diagnostics)
                .map(|(severity, message)| ConnectionConversionMenuPlan::Reject(severity, message))
                .unwrap_or(ConnectionConversionMenuPlan::Ignore),
        }
    }

    pub(super) fn plan_connection_conversion_menu_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        candidate: &InsertNodeCandidate,
    ) -> ConnectionConversionMenuPlan {
        let zoom = self.cached_zoom;
        let style = self.style.clone();
        let presenter = &mut *self.presenter;
        self.graph
            .read_ref(host, |graph| {
                Self::plan_connection_conversion_menu_candidate_with_graph(
                    presenter, graph, &style, zoom, from, to, at, candidate,
                )
            })
            .ok()
            .unwrap_or(ConnectionConversionMenuPlan::Ignore)
    }

    pub(super) fn activate_connection_conversion_picker_action<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        from: PortId,
        to: PortId,
        at: CanvasPoint,
        invoked_at: Point,
        action: NodeGraphContextMenuAction,
        menu_candidates: &[InsertNodeCandidate],
    ) -> bool {
        match action {
            NodeGraphContextMenuAction::InsertNodeCandidate(candidate_ix) => {
                let Some(candidate) = menu_candidates.get(candidate_ix).cloned() else {
                    return true;
                };
                self.record_recent_kind(&candidate.kind);
                let plan = self
                    .plan_connection_conversion_menu_candidate(cx.app, from, to, at, &candidate);
                self.apply_connection_conversion_menu_plan(cx, from, invoked_at, plan);
                true
            }
            _ => false,
        }
    }

    pub(super) fn apply_connection_conversion_menu_plan<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
        plan: ConnectionConversionMenuPlan,
    ) {
        match plan {
            ConnectionConversionMenuPlan::Apply(ops) => {
                let node_id = Self::first_added_node_id(&ops);
                self.apply_ops(cx.app, cx.window, ops);
                self.interaction.suspended_wire_drag = None;
                self.select_inserted_node(cx.app, node_id);
            }
            ConnectionConversionMenuPlan::Reject(severity, message) => {
                self.show_toast(cx.app, cx.window, severity, message);
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
            ConnectionConversionMenuPlan::Ignore => {
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
        }
    }
}
