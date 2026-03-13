use super::*;

pub(super) fn resolve_edge_anchor_target_from_render(
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
