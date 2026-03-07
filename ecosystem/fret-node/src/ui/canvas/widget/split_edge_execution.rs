use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_split_edge_reroute_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        edge_id: EdgeId,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Vec<Diagnostic>> {
        let plan = presenter.plan_split_edge(graph, edge_id, &NodeKindKey::new(REROUTE_KIND), at);
        match plan.decision {
            ConnectDecision::Accept => Ok(plan.ops),
            ConnectDecision::Reject => Err(plan.diagnostics),
        }
    }

    pub(super) fn plan_canvas_split_edge_reroute<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
        invoked_at: Point,
    ) -> Option<Result<Vec<GraphOp>, Vec<Diagnostic>>> {
        let at = self.reroute_pos_for_invoked_at(invoked_at);
        let presenter = &mut *self.presenter;
        self.graph
            .read_ref(host, |graph| {
                Self::plan_split_edge_reroute_with_graph(presenter, graph, edge_id, at)
            })
            .ok()
    }

    pub(super) fn split_edge_reroute_rejection_toast(
        diags: &[Diagnostic],
    ) -> (DiagnosticSeverity, Arc<str>) {
        Self::toast_from_diagnostics(diags).unwrap_or_else(|| {
            (
                DiagnosticSeverity::Error,
                Arc::<str>::from("failed to insert reroute"),
            )
        })
    }

    pub(super) fn apply_split_edge_reroute_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        label: Option<&str>,
        ops: Vec<GraphOp>,
    ) -> bool {
        let node_id = Self::first_added_node_id(&ops);
        let applied = self.commit_ops(host, window, label, ops);
        if applied {
            self.select_inserted_node(host, node_id);
        }
        applied
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{DiagnosticSeverity, DiagnosticTarget};

    #[test]
    fn split_edge_reroute_rejection_toast_uses_first_diagnostic_message() {
        let toast = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::split_edge_reroute_rejection_toast(&[
            Diagnostic {
                key: "reroute_rejected".into(),
                severity: DiagnosticSeverity::Warning,
                target: DiagnosticTarget::Graph,
                message: "reroute was rejected".into(),
                fixes: Vec::new(),
            },
        ]);

        assert_eq!(toast.0, DiagnosticSeverity::Warning);
        assert_eq!(&*toast.1, "reroute was rejected");
    }

    #[test]
    fn split_edge_reroute_rejection_toast_falls_back_when_message_missing() {
        let toast = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::split_edge_reroute_rejection_toast(&[
            Diagnostic {
                key: "reroute_rejected".into(),
                severity: DiagnosticSeverity::Info,
                target: DiagnosticTarget::Graph,
                message: String::new(),
                fixes: Vec::new(),
            },
        ]);

        assert_eq!(toast.0, DiagnosticSeverity::Error);
        assert_eq!(&*toast.1, "failed to insert reroute");
    }
}
