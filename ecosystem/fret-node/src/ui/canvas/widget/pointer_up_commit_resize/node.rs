use crate::core::Graph;
use crate::ops::GraphOp;
use crate::ui::canvas::state::NodeResize;

pub(in super::super) fn build_node_resize_ops(resize: &NodeResize, graph: &Graph) -> Vec<GraphOp> {
    let mut ops = node_resize_ops(resize);
    ops.extend(node_resize_group_rect_ops(resize, graph));
    ops
}

fn node_resize_ops(resize: &NodeResize) -> Vec<GraphOp> {
    let mut ops: Vec<GraphOp> = Vec::new();
    if resize.start_node_pos != resize.current_node_pos {
        ops.push(GraphOp::SetNodePos {
            id: resize.node,
            from: resize.start_node_pos,
            to: resize.current_node_pos,
        });
    }
    if resize.start_size_opt != resize.current_size_opt {
        ops.push(GraphOp::SetNodeSize {
            id: resize.node,
            from: resize.start_size_opt,
            to: resize.current_size_opt,
        });
    }
    ops
}

fn node_resize_group_rect_ops(resize: &NodeResize, graph: &Graph) -> Vec<GraphOp> {
    resize
        .current_groups
        .iter()
        .filter_map(|(id, to)| {
            let from = graph.groups.get(id).map(|group| group.rect)?;
            (from != *to).then_some(GraphOp::SetGroupRect {
                id: *id,
                from,
                to: *to,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
