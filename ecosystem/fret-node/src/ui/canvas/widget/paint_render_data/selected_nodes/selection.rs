use crate::ui::canvas::widget::*;

pub(super) fn selected_nodes(snapshot: &ViewSnapshot) -> Vec<GraphNodeId> {
    snapshot.selected_nodes.clone()
}
