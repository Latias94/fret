use std::collections::HashMap;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_edge_labels_tail<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    render: &RenderData,
    custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
    bezier_steps: usize,
    zoom: f32,
    view_interacting: bool,
) {
    if !render.edges.iter().any(|edge| {
        edge.hint
            .label
            .as_ref()
            .is_some_and(|label| !label.is_empty())
    }) {
        return;
    }

    let label_budget_limit =
        NodeGraphCanvasWith::<M>::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
    let mut label_budget = WorkBudget::new(label_budget_limit);
    let (next_edge, skipped_by_budget) = canvas.paint_edge_labels_static_budgeted(
        cx.scene,
        cx.services,
        cx.scale_factor,
        &render.edges,
        (!custom_paths.is_empty()).then_some(custom_paths),
        bezier_steps,
        zoom,
        0,
        &mut label_budget,
    );
    let mut label_budget_skipped: u32 = 0;
    if skipped_by_budget && next_edge < render.edges.len() {
        label_budget_skipped = 1;
        super::super::super::redraw_request::request_paint_redraw(cx);
    }

    super::stats::record_edge_budget_stat(
        cx,
        "fret-node.canvas.edge_labels_budget",
        label_budget_limit,
        label_budget.used(),
        label_budget_skipped,
    );
}
