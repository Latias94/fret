use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct RejectNonFiniteTx;

impl NodeGraphCanvasMiddleware for RejectNonFiniteTx {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        let Some((key, message)) = crate::ops::find_non_finite_in_tx(tx) else {
            return NodeGraphCanvasCommitOutcome::Continue;
        };

        NodeGraphCanvasCommitOutcome::Reject {
            diagnostics: vec![Diagnostic {
                key,
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RejectInvalidSizeTx;

impl NodeGraphCanvasMiddleware for RejectInvalidSizeTx {
    fn before_commit<H: UiHost>(
        &mut self,
        _host: &mut H,
        _window: Option<AppWindowId>,
        _ctx: &NodeGraphCanvasMiddlewareCx<'_>,
        tx: &mut GraphTransaction,
    ) -> NodeGraphCanvasCommitOutcome {
        let Some((key, message)) = crate::ops::find_invalid_size_in_tx(tx) else {
            return NodeGraphCanvasCommitOutcome::Continue;
        };

        NodeGraphCanvasCommitOutcome::Reject {
            diagnostics: vec![Diagnostic {
                key,
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message,
                fixes: Vec::new(),
            }],
        }
    }
}
