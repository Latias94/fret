use crate::ui::canvas::widget::*;

pub(super) fn split_edge_reroute_rejection_toast<M: NodeGraphCanvasMiddleware>(
    diags: &[Diagnostic],
) -> (DiagnosticSeverity, Arc<str>) {
    NodeGraphCanvasWith::<M>::toast_from_diagnostics(diags).unwrap_or_else(|| {
        (
            DiagnosticSeverity::Error,
            Arc::<str>::from("failed to insert reroute"),
        )
    })
}
