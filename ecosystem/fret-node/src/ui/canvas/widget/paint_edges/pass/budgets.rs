use crate::ui::canvas::widget::*;

use super::EdgePaintBudgets;

pub(super) fn new_edge_paint_budgets<M: NodeGraphCanvasMiddleware>(
    view_interacting: bool,
) -> EdgePaintBudgets {
    let marker_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let outline_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let highlight_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME
            .select(view_interacting);
    EdgePaintBudgets {
        marker_budget_limit,
        marker_budget: WorkBudget::new(marker_budget_limit),
        marker_budget_skipped: 0,
        wire_budget: WorkBudget::new(u32::MAX / 2),
        outline_budget: WorkBudget::new(outline_budget_limit),
        outline_budget_skipped: 0,
        highlight_budget: WorkBudget::new(highlight_budget_limit),
        highlight_budget_skipped: 0,
    }
}
