use super::super::*;

pub(in super::super) fn push_group_remove_ops(
    ops: &mut Vec<GraphOp>,
    graph: &Graph,
    selected_groups: &[crate::core::GroupId],
) {
    let mut groups: Vec<crate::core::GroupId> = selected_groups.to_vec();
    groups.sort();
    for group_id in groups {
        if let Some(op) = graph.build_remove_group_op(group_id) {
            ops.push(op);
        }
    }
}
