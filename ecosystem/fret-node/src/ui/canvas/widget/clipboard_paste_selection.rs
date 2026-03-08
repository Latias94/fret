use super::*;

#[derive(Default)]
pub(super) struct InsertedEntities {
    nodes: Vec<GraphNodeId>,
    groups: Vec<crate::core::GroupId>,
}

pub(super) fn inserted_entities(tx: &GraphTransaction) -> InsertedEntities {
    InsertedEntities {
        nodes: tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddNode { id, .. } => Some(*id),
                _ => None,
            })
            .collect(),
        groups: tx
            .ops
            .iter()
            .filter_map(|op| match op {
                GraphOp::AddGroup { id, .. } => Some(*id),
                _ => None,
            })
            .collect(),
    }
}

pub(super) fn apply_inserted_selection<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    inserted: InsertedEntities,
) {
    if inserted.nodes.is_empty() && inserted.groups.is_empty() {
        return;
    }

    canvas.update_view_state(host, |state| {
        state.selected_edges.clear();
        state.selected_nodes = inserted.nodes.clone();
        state.selected_groups = inserted.groups.clone();
        for id in &inserted.nodes {
            state.draw_order.retain(|x| x != id);
            state.draw_order.push(*id);
        }
        for id in &inserted.groups {
            state.group_draw_order.retain(|x| x != id);
            state.group_draw_order.push(*id);
        }
    });
}
