use std::collections::HashMap;

use crate::ui::canvas::widget::*;

use super::super::prepare::EdgePaint;
use super::EdgePaintBudgets;

#[allow(clippy::too_many_arguments)]
pub(super) fn paint_edge_batches<M: NodeGraphCanvasMiddleware, H: UiHost>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
    edges: impl IntoIterator<Item = EdgePaint>,
    custom_paths: &HashMap<EdgeId, crate::ui::edge_types::EdgeCustomPath>,
    interaction_hint: crate::ui::InteractionChromeHint,
    zoom: f32,
    budgets: &mut EdgePaintBudgets,
) {
    for edge in edges {
        let custom = custom_paths.get(&edge.id);
        let chrome = canvas.prepare_edge_chrome(
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
            canvas.push_edge_custom_wire_and_markers_budgeted(
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
            canvas.push_edge_wire_and_markers_budgeted(
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
