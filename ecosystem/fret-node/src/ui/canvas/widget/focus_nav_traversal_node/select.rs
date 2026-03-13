use crate::ui::canvas::widget::*;

pub(super) fn next_node(
    ordered: &[GraphNodeId],
    current: Option<GraphNodeId>,
    forward: bool,
) -> Option<GraphNodeId> {
    if ordered.is_empty() {
        return None;
    }

    match current.and_then(|id| ordered.iter().position(|entry| *entry == id)) {
        Some(index) => {
            let len = ordered.len();
            let next_index = if forward {
                (index + 1) % len
            } else {
                (index + len - 1) % len
            };
            Some(ordered[next_index])
        }
        None => Some(if forward {
            ordered[0]
        } else {
            ordered[ordered.len() - 1]
        }),
    }
}
