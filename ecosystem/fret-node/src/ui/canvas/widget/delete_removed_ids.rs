use super::*;

pub(super) fn removed_ids_from_ops(
    ops: &[GraphOp],
) -> (
    HashSet<GraphNodeId>,
    HashSet<EdgeId>,
    HashSet<crate::core::GroupId>,
) {
    let mut removed_nodes: HashSet<GraphNodeId> = HashSet::new();
    let mut removed_edges: HashSet<EdgeId> = HashSet::new();
    let mut removed_groups: HashSet<crate::core::GroupId> = HashSet::new();

    for op in ops {
        match op {
            GraphOp::RemoveNode { id, edges, .. } => {
                removed_nodes.insert(*id);
                for (edge_id, _) in edges {
                    removed_edges.insert(*edge_id);
                }
            }
            GraphOp::RemoveEdge { id, .. } => {
                removed_edges.insert(*id);
            }
            GraphOp::RemoveGroup { id, .. } => {
                removed_groups.insert(*id);
            }
            _ => {}
        }
    }

    (removed_nodes, removed_edges, removed_groups)
}
