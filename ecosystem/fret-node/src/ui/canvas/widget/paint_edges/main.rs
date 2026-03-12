#[path = "main/context.rs"]
mod context;
#[path = "main/markers.rs"]
mod markers;

use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::pass::EdgePaintBudgets;
use super::prepare::PreparedEdgePaintBatches;

struct PreparedEdgePaintFrame {
    interaction_hint: crate::ui::InteractionChromeHint,
    custom_paths: std::collections::HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
    bezier_steps: usize,
    batches: PreparedEdgePaintBatches,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn paint_edges<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        render: &RenderData,
        geom: &CanvasGeometry,
        zoom: f32,
        view_interacting: bool,
    ) {
        let PreparedEdgePaintFrame {
            interaction_hint,
            custom_paths,
            bezier_steps,
            batches:
                PreparedEdgePaintBatches {
                    edges_normal,
                    edges_selected,
                    edges_hovered,
                    edge_insert_marker,
                    insert_node_drag_marker,
                },
        } = context::prepare_edge_paint_frame(self, cx, snapshot, render, zoom);

        let mut budgets = EdgePaintBudgets::new(self, view_interacting);
        self.paint_edge_batches(
            cx,
            edges_normal
                .into_iter()
                .chain(edges_selected)
                .chain(edges_hovered),
            &custom_paths,
            interaction_hint,
            zoom,
            &mut budgets,
        );
        Self::request_edge_budget_redraws(cx, &budgets);

        markers::push_optional_drop_marker(cx.scene, edge_insert_marker, zoom);
        markers::push_optional_drop_marker(cx.scene, insert_node_drag_marker, zoom);

        self.paint_edge_labels_tail(
            cx,
            render,
            &custom_paths,
            bezier_steps,
            zoom,
            view_interacting,
        );
        Self::record_edge_marker_budget_stat(
            cx,
            budgets.marker_budget_limit,
            budgets.marker_budget.used(),
            budgets.marker_budget_skipped,
        );

        self.paint_wire_drag_preview(
            cx,
            render,
            geom,
            zoom,
            interaction_hint,
            &mut budgets.outline_budget,
            &mut budgets.outline_budget_skipped,
        );
    }
}
