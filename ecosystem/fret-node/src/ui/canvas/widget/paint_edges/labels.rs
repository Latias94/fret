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
        if !render.edges.iter().any(|edge| {
            edge.hint
                .label
                .as_ref()
                .is_some_and(|label| !label.is_empty())
        }) {
            return;
        }

        let label_budget_limit = Self::EDGE_LABEL_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let mut label_budget = WorkBudget::new(label_budget_limit);
        let (next_edge, skipped_by_budget) = self.paint_edge_labels_static_budgeted(
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
            super::super::redraw_request::request_paint_redraw(cx);
        }

        Self::record_edge_budget_stat(
            cx,
            "fret-node.canvas.edge_labels_budget",
            label_budget_limit,
            label_budget.used(),
            label_budget_skipped,
        );
    }

    pub(super) fn record_edge_marker_budget_stat<H: UiHost>(
        cx: &mut PaintCx<'_, H>,
        marker_budget_limit: u32,
        marker_budget_used: u32,
        marker_budget_skipped: u32,
    ) {
        Self::record_edge_budget_stat(
            cx,
            "fret-node.canvas.edge_markers_budget",
            marker_budget_limit,
            marker_budget_used,
            marker_budget_skipped,
        );
    }

    fn record_edge_budget_stat<H: UiHost>(
        cx: &mut PaintCx<'_, H>,
        name: &'static str,
        limit: u32,
        used: u32,
        skipped: u32,
    ) {
        let Some(window) = cx.window else {
            return;
        };
        let frame_id = cx.app.frame_id().0;
        let key = CanvasCacheKey {
            window: window.data().as_ffi(),
            node: cx.node.data().as_ffi(),
            name,
        };
        cx.app
            .with_global_mut(CanvasCacheStatsRegistry::default, |registry, _app| {
                registry.record_work_budget(
                    key,
                    frame_id,
                    used.saturating_add(skipped),
                    limit,
                    used,
                    skipped,
                );
            });
    }
}
