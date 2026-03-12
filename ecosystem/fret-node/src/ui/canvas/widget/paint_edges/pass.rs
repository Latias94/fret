#[path = "pass/batch.rs"]
mod batch;
#[path = "pass/budgets.rs"]
mod budgets;
#[path = "pass/redraw.rs"]
mod redraw;

use std::collections::HashMap;

use crate::ui::canvas::widget::*;

use super::prepare::EdgePaint;

pub(super) struct EdgePaintBudgets {
    pub marker_budget_limit: u32,
    pub marker_budget: WorkBudget,
    pub marker_budget_skipped: u32,
    pub wire_budget: WorkBudget,
    pub outline_budget: WorkBudget,
    pub outline_budget_skipped: u32,
    pub highlight_budget: WorkBudget,
    pub highlight_budget_skipped: u32,
}

impl EdgePaintBudgets {
    pub(super) fn new<M: NodeGraphCanvasMiddleware>(
        _widget: &NodeGraphCanvasWith<M>,
        view_interacting: bool,
    ) -> Self {
        budgets::new_edge_paint_budgets::<M>(view_interacting)
    }
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_edge_batches<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        edges: impl IntoIterator<Item = EdgePaint>,
        custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
        interaction_hint: crate::ui::InteractionChromeHint,
        zoom: f32,
        budgets: &mut EdgePaintBudgets,
    ) {
        batch::paint_edge_batches(
            self,
            cx,
            edges,
            custom_paths,
            interaction_hint,
            zoom,
            budgets,
        );
    }

    pub(super) fn request_edge_budget_redraws<H: UiHost>(
        cx: &mut PaintCx<'_, H>,
        budgets: &EdgePaintBudgets,
    ) {
        redraw::request_edge_budget_redraws(cx, budgets);
    }
}
