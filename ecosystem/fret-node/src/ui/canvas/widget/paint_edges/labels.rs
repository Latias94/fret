#[path = "labels/stats.rs"]
mod stats;
#[path = "labels/tail.rs"]
mod tail;

use std::collections::HashMap;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_edge_labels_tail<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        render: &RenderData,
        custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
        bezier_steps: usize,
        zoom: f32,
        view_interacting: bool,
    ) {
        tail::paint_edge_labels_tail(
            self,
            cx,
            render,
            custom_paths,
            bezier_steps,
            zoom,
            view_interacting,
        );
    }

    pub(super) fn record_edge_marker_budget_stat<H: UiHost>(
        cx: &mut PaintCx<'_, H>,
        marker_budget_limit: u32,
        marker_budget_used: u32,
        marker_budget_skipped: u32,
    ) {
        stats::record_edge_budget_stat(
            cx,
            "fret-node.canvas.edge_markers_budget",
            marker_budget_limit,
            marker_budget_used,
            marker_budget_skipped,
        );
    }
}
