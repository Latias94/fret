use super::super::*;

pub(super) fn visible_node_ids_for_render(
    geom: &CanvasGeometry,
    index: &CanvasSpatialDerived,
    cull: Option<Rect>,
) -> (usize, Vec<GraphNodeId>) {
    if let Some(cull_rect) = cull {
        let mut candidates: Vec<GraphNodeId> = Vec::new();
        index.query_nodes_in_rect(cull_rect, &mut candidates);

        let mut visible: Vec<GraphNodeId> = Vec::with_capacity(candidates.len());
        for node in candidates.iter().copied() {
            let Some(node_geom) = geom.nodes.get(&node) else {
                continue;
            };
            if rects_intersect(node_geom.rect, cull_rect) {
                visible.push(node);
            }
        }

        visible.sort_unstable_by_key(|node| {
            (geom.node_rank.get(node).copied().unwrap_or(u32::MAX), *node)
        });

        (candidates.len(), visible)
    } else {
        (geom.order.len(), geom.order.clone())
    }
}
