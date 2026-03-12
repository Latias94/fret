use super::super::*;

pub(super) fn candidate_edge_ids_for_render(
    graph: &Graph,
    index: &CanvasSpatialDerived,
    cull: Option<Rect>,
) -> Vec<EdgeId> {
    let mut edge_ids: Vec<EdgeId> = Vec::new();
    if let Some(c) = cull {
        index.query_edges_in_rect(c, &mut edge_ids);
    } else {
        edge_ids.extend(graph.edges.keys().copied());
    }
    edge_ids
}

pub(super) fn edge_rank_for_render(geom: &CanvasGeometry, edge: &Edge) -> u32 {
    geom.ports
        .get(&edge.from)
        .and_then(|p| geom.node_rank.get(&p.node).copied())
        .unwrap_or(0)
        .max(
            geom.ports
                .get(&edge.to)
                .and_then(|p| geom.node_rank.get(&p.node).copied())
                .unwrap_or(0),
        )
}
