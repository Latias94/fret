use crate::ui::canvas::widget::paint_render_data::RenderData;
use crate::ui::canvas::widget::*;

pub(super) type EdgeAnchorTarget = (EdgeRouteKind, Point, Point, Color);

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn resolve_edge_anchor_target_id<H: UiHost>(
        &self,
        cx: &PaintCx<'_, H>,
        snapshot: &ViewSnapshot,
    ) -> Option<EdgeId> {
        self.interaction
            .focused_edge
            .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
            .filter(|edge_id| {
                self.graph
                    .read_ref(cx.app, |graph| {
                        let edge = graph.edges.get(edge_id)?;
                        let (allow_source, allow_target) =
                            Self::edge_reconnectable_flags(edge, &snapshot.interaction);
                        Some(allow_source || allow_target)
                    })
                    .ok()
                    .flatten()
                    .unwrap_or(false)
            })
    }

    pub(super) fn resolve_edge_anchor_target_from_render(
        &self,
        render: &RenderData,
        edge_id: Option<EdgeId>,
    ) -> Option<EdgeAnchorTarget> {
        let edge_id = edge_id?;
        render
            .edges
            .iter()
            .find(|edge| edge.id == edge_id)
            .map(|edge| (edge.hint.route, edge.from, edge.to, edge.color))
    }

    pub(super) fn resolve_edge_anchor_target_from_geometry<H: UiHost>(
        &self,
        cx: &PaintCx<'_, H>,
        geom: &CanvasGeometry,
        edge_id: Option<EdgeId>,
    ) -> Option<EdgeAnchorTarget> {
        let edge_id = edge_id?;
        self.graph
            .read_ref(cx.app, |graph| {
                let edge = graph.edges.get(&edge_id)?;
                let from = geom.port_center(edge.from)?;
                let to = geom.port_center(edge.to)?;
                let hint =
                    EdgePathContext::new(&self.style, &*self.presenter, self.edge_types.as_ref())
                        .edge_render_hint_normalized(graph, edge_id);
                let mut color = self.presenter.edge_color(graph, edge_id, &self.style);
                if let Some(override_color) = hint.color {
                    color = override_color;
                }
                Some((hint.route, from, to, color))
            })
            .ok()
            .flatten()
    }
}
