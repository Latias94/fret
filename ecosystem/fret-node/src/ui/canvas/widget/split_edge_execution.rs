mod apply;
mod execute;
mod plan;
mod toast;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn plan_canvas_split_edge_reroute<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
        invoked_at: Point,
    ) -> Option<Result<Vec<GraphOp>, Vec<Diagnostic>>> {
        plan::plan_canvas_split_edge_reroute(self, host, edge_id, invoked_at)
    }

    pub(super) fn apply_split_edge_reroute_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        label: Option<&str>,
        ops: Vec<GraphOp>,
    ) -> bool {
        apply::apply_split_edge_reroute_ops(self, host, window, label, ops)
    }

    pub(super) fn execute_split_edge_reroute_outcome<H: UiHost>(
        &mut self,
        host: &mut H,
        window: Option<AppWindowId>,
        label: Option<&str>,
        outcome: Option<Result<Vec<GraphOp>, Vec<Diagnostic>>>,
    ) -> bool {
        execute::execute_split_edge_reroute_outcome(self, host, window, label, outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{DiagnosticSeverity, DiagnosticTarget};

    #[test]
    fn split_edge_reroute_rejection_toast_uses_first_diagnostic_message() {
        let toast = toast::split_edge_reroute_rejection_toast::<NoopNodeGraphCanvasMiddleware>(&[
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
        let toast = toast::split_edge_reroute_rejection_toast::<NoopNodeGraphCanvasMiddleware>(&[
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
