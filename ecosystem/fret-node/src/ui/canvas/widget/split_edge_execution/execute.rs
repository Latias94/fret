use crate::ui::canvas::widget::*;

use super::toast;

pub(super) fn execute_split_edge_reroute_outcome<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    label: Option<&str>,
    outcome: Option<Result<Vec<GraphOp>, Vec<Diagnostic>>>,
) -> bool {
    match outcome {
        Some(Ok(ops)) => canvas.apply_split_edge_reroute_ops(host, window, label, ops),
        Some(Err(diags)) => {
            let (sev, msg) = toast::split_edge_reroute_rejection_toast::<M>(&diags);
            canvas.show_toast(host, window, sev, msg);
            false
        }
        None => false,
    }
}
