use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

use super::pass::EdgePaintBudgets;
use super::prepare::PreparedEdgePaintBatches;
use super::preview::push_drop_marker;

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
        let interaction_hint = if let Some(skin) = self.skin.as_ref() {
            self.graph
                .read_ref(cx.app, |g| skin.interaction_chrome_hint(g, &self.style))
                .ok()
                .unwrap_or_default()
        } else {
            crate::ui::InteractionChromeHint::default()
        };

        let custom_paths = self.collect_custom_edge_paths(&*cx.app, &render.edges, zoom);
        let bezier_steps = usize::from(snapshot.interaction.bezier_hit_test_steps.max(1));
        let PreparedEdgePaintBatches {
            edges_normal,
            edges_selected,
            edges_hovered,
            edge_insert_marker,
            insert_node_drag_marker,
        } = self.prepare_edge_paint_batches(snapshot, render, &custom_paths, zoom);

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

        if let Some((pos, color)) = edge_insert_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }
        if let Some((pos, color)) = insert_node_drag_marker {
            push_drop_marker(cx.scene, pos, color, zoom);
        }

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
