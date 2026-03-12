mod render;
mod resolve;
mod state;
mod style;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn paint_edge_focus_anchors<H: UiHost>(
        &mut self,
        cx: &mut PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
        edge_anchor_target_id: Option<EdgeId>,
        edge_anchor_target: Option<(EdgeRouteKind, Point, Point, Color)>,
        zoom: f32,
    ) {
        let Some((route, from, to, color)) = edge_anchor_target else {
            return;
        };

        let (a0, a1) = Self::edge_focus_anchor_centers(route, from, to, zoom);
        let target_edge_id = edge_anchor_target_id;
        let reconnectable =
            resolve::target_edge_reconnectable_flags(self, cx.app, snapshot, target_edge_id);
        let base_style = style::base_anchor_style::<M>(color, zoom);

        for (endpoint, center) in [(EdgeEndpoint::From, a0), (EdgeEndpoint::To, a1)] {
            if !resolve::edge_anchor_endpoint_allowed(endpoint, reconnectable) {
                continue;
            }

            let rect = Self::edge_focus_anchor_rect(center, zoom);
            let interaction_state =
                state::edge_anchor_interaction_state(&self.interaction, target_edge_id, endpoint);
            let paint_style = style::edge_anchor_paint_style(base_style, interaction_state);

            render::push_edge_focus_anchor_quad(cx, rect, paint_style);
        }
    }
}
