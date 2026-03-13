#[path = "edges/append.rs"]
mod append;
#[path = "edges/candidate.rs"]
mod candidate;
#[path = "edges/cull.rs"]
mod cull;
#[path = "edges/hint.rs"]
mod hint;

use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_edge_render_data(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        geom: &CanvasGeometry,
        index: &CanvasSpatialDerived,
        presenter: &dyn NodeGraphPresenter,
        selected_edges: &HashSet<EdgeId>,
        hovered_edge: Option<EdgeId>,
        cull: Option<Rect>,
        zoom: f32,
        out: &mut RenderData,
    ) {
        out.metrics.edge_total = graph.edges.len();
        let edge_ids = candidate::candidate_edge_ids_for_render(graph, index, cull);
        out.metrics.edge_candidates = edge_ids.len();

        for edge_id in edge_ids {
            append::append_edge_render_data(
                self,
                graph,
                snapshot,
                geom,
                presenter,
                selected_edges,
                hovered_edge,
                cull,
                zoom,
                out,
                edge_id,
            );
        }

        out.edges
            .sort_unstable_by(|a, b| a.rank.cmp(&b.rank).then_with(|| a.id.cmp(&b.id)));
    }
}
