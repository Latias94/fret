use crate::ui::canvas::widget::*;

#[derive(Debug)]
pub(super) enum ConnectionInsertMenuPlan {
    Apply(workflow::WireDropInsertPlan),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

#[derive(Debug)]
pub(super) enum ConnectionConversionMenuPlan {
    Apply(Vec<GraphOp>),
    Reject(DiagnosticSeverity, Arc<str>),
    Ignore,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
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
                    if let Some((sev, msg)) = toast {
                        self.show_toast(cx.app, cx.window, sev, msg);
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
            ConnectionInsertMenuPlan::Reject(sev, msg) => {
                self.show_toast(cx.app, cx.window, sev, msg);
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

    fn restore_connection_menu_wire_drag<H: UiHost>(
        &mut self,
        cx: &mut EventCx<'_, H>,
        fallback_from: PortId,
        invoked_at: Point,
    ) {
        let resume_pos = self.interaction.last_pos.unwrap_or(invoked_at);
        self.restore_suspended_wire_drag(cx, Some(fallback_from), resume_pos);
    }

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
                .map(|(sev, msg)| ConnectionConversionMenuPlan::Reject(sev, msg))
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
            ConnectionConversionMenuPlan::Reject(sev, msg) => {
                self.show_toast(cx.app, cx.window, sev, msg);
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
            ConnectionConversionMenuPlan::Ignore => {
                self.restore_connection_menu_wire_drag(cx, fallback_from, invoked_at);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Graph, GraphId, PortId};
    use crate::ui::{DefaultNodeGraphPresenter, NodeGraphStyle};
    use serde_json::Value;

    fn regular_candidate() -> InsertNodeCandidate {
        InsertNodeCandidate {
            kind: NodeKindKey::new("regular"),
            label: Arc::<str>::from("Regular"),
            enabled: true,
            template: None,
            payload: Value::Null,
        }
    }

    #[test]
    fn connection_insert_menu_plan_surfaces_create_node_errors() {
        let mut presenter = DefaultNodeGraphPresenter::default();
        let graph = Graph::new(GraphId::new());
        let candidate = regular_candidate();
        let plan = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::plan_connection_insert_menu_candidate_with_graph(
            &mut presenter,
            &graph,
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            NodeGraphConnectionMode::Strict,
            &candidate,
        );

        assert!(matches!(
            plan,
            ConnectionInsertMenuPlan::Reject(DiagnosticSeverity::Info, ref msg)
                if &**msg == "node insertion is not supported"
        ));
    }

    #[test]
    fn connection_conversion_menu_plan_rejects_missing_template() {
        let mut presenter = DefaultNodeGraphPresenter::default();
        let graph = Graph::new(GraphId::new());
        let candidate = regular_candidate();
        let plan = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::plan_connection_conversion_menu_candidate_with_graph(
            &mut presenter,
            &graph,
            &NodeGraphStyle::default(),
            1.0,
            PortId::new(),
            PortId::new(),
            CanvasPoint { x: 10.0, y: 20.0 },
            &candidate,
        );

        assert!(matches!(
            plan,
            ConnectionConversionMenuPlan::Reject(DiagnosticSeverity::Error, ref msg)
                if &**msg == "conversion candidate is missing template"
        ));
    }
}
