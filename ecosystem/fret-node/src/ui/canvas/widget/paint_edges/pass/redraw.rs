use crate::ui::canvas::widget::*;

use super::EdgePaintBudgets;

pub(super) fn request_edge_budget_redraws<H: UiHost>(
    cx: &mut PaintCx<'_, H>,
    budgets: &EdgePaintBudgets,
) {
    super::super::super::redraw_request::request_paint_redraw_if(
        cx,
        budgets.marker_budget_skipped > 0,
    );
    super::super::super::redraw_request::request_paint_redraw_if(
        cx,
        budgets.outline_budget_skipped > 0,
    );
    super::super::super::redraw_request::request_paint_redraw_if(
        cx,
        budgets.highlight_budget_skipped > 0,
    );
}
