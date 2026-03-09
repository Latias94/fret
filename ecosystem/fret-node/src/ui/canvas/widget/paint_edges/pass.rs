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
        let marker_budget_limit =
            NodeGraphCanvasWith::<M>::EDGE_MARKER_BUILD_BUDGET_PER_FRAME.select(view_interacting);
        let outline_budget_limit =
            NodeGraphCanvasWith::<M>::EDGE_WIRE_OUTLINE_BUILD_BUDGET_PER_FRAME
                .select(view_interacting);
        let highlight_budget_limit =
            NodeGraphCanvasWith::<M>::EDGE_WIRE_HIGHLIGHT_BUILD_BUDGET_PER_FRAME
                .select(view_interacting);
        Self {
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
        for edge in edges {
            let custom = custom_paths.get(&edge.id);
            let chrome = self.prepare_edge_chrome(
                cx,
                custom,
                interaction_hint,
                edge.selected,
                edge.hovered,
                edge.id,
                edge.from,
                edge.to,
                edge.route,
                edge.color,
                edge.width,
                edge.dash,
                zoom,
                &mut budgets.outline_budget,
                &mut budgets.outline_budget_skipped,
                &mut budgets.highlight_budget,
                &mut budgets.highlight_budget_skipped,
            );

            let (_stop, skipped) = if let Some(custom) = custom {
                let fallback = Point::new(
                    Px(edge.to.x.0 - edge.from.x.0),
                    Px(edge.to.y.0 - edge.from.y.0),
                );
                let (t0, t1) =
                    path_start_end_tangents(&custom.commands).unwrap_or((fallback, fallback));
                self.push_edge_custom_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    custom.cache_key,
                    &custom.commands,
                    t0,
                    t1,
                    zoom,
                    cx.scale_factor,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    chrome.highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut budgets.wire_budget,
                    &mut budgets.marker_budget,
                    false,
                )
            } else {
                self.push_edge_wire_and_markers_budgeted(
                    cx.scene,
                    cx.services,
                    zoom,
                    cx.scale_factor,
                    edge.route,
                    edge.from,
                    edge.to,
                    edge.paint,
                    edge.color,
                    edge.width,
                    edge.dash,
                    chrome.highlight,
                    edge.start_marker.as_ref(),
                    edge.end_marker.as_ref(),
                    &mut budgets.wire_budget,
                    &mut budgets.marker_budget,
                    false,
                )
            };
            budgets.marker_budget_skipped = budgets.marker_budget_skipped.saturating_add(skipped);

            if chrome.glow_pushed {
                cx.scene.push(SceneOp::PopEffect);
            }
        }
    }

    pub(super) fn request_edge_budget_redraws<H: UiHost>(
        cx: &mut PaintCx<'_, H>,
        budgets: &EdgePaintBudgets,
    ) {
        super::super::redraw_request::request_paint_redraw_if(
            cx,
            budgets.marker_budget_skipped > 0,
        );
        super::super::redraw_request::request_paint_redraw_if(
            cx,
            budgets.outline_budget_skipped > 0,
        );
        super::super::redraw_request::request_paint_redraw_if(
            cx,
            budgets.highlight_budget_skipped > 0,
        );
    }
}
