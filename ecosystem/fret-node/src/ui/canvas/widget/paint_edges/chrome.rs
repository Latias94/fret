#[path = "chrome/glow.rs"]
mod glow;
#[path = "chrome/highlight.rs"]
mod highlight;
#[path = "chrome/outline.rs"]
mod outline;

use crate::ui::canvas::widget::*;
use fret_core::scene::DashPatternV1;

use super::markers::WireHighlightPaint;

#[derive(Debug, Clone, Copy)]
pub(super) struct EdgeChromeState {
    pub glow_pushed: bool,
    pub highlight: Option<WireHighlightPaint>,
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn prepare_edge_chrome<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        custom: Option<&crate::ui::edge_types::EdgeCustomPath>,
        interaction_hint: crate::ui::InteractionChromeHint,
        edge_selected: bool,
        edge_hovered: bool,
        edge_id: EdgeId,
        from: Point,
        to: Point,
        route: EdgeRouteKind,
        color: Color,
        width: f32,
        dash: Option<DashPatternV1>,
        zoom: f32,
        outline_budget: &mut WorkBudget,
        outline_budget_skipped: &mut u32,
        highlight_budget: &mut WorkBudget,
        highlight_budget_skipped: &mut u32,
    ) -> EdgeChromeState {
        outline::push_edge_outline(
            self,
            cx,
            custom,
            interaction_hint,
            edge_selected,
            edge_id,
            from,
            to,
            route,
            width,
            dash,
            zoom,
            outline_budget,
            outline_budget_skipped,
        );

        let glow_pushed = glow::push_edge_glow(
            self,
            cx,
            custom,
            interaction_hint,
            edge_selected,
            route,
            from,
            to,
            color,
            width,
            zoom,
        );

        let highlight = highlight::resolve_edge_highlight(
            interaction_hint,
            edge_selected,
            edge_hovered,
            color,
            width,
            highlight_budget,
            highlight_budget_skipped,
        );

        EdgeChromeState {
            glow_pushed,
            highlight,
        }
    }
}
