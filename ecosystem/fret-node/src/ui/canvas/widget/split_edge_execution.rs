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
mod tests;
