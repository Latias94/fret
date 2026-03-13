use std::collections::HashSet;

use crate::ui::canvas::widget::*;

pub(super) struct RenderSelections {
    pub(super) selected_nodes: HashSet<GraphNodeId>,
    pub(super) selected_edges: HashSet<EdgeId>,
    pub(super) selected_groups: HashSet<crate::core::GroupId>,
}

pub(super) fn collect_render_selections(snapshot: &ViewSnapshot) -> RenderSelections {
    RenderSelections {
        selected_nodes: snapshot.selected_nodes.iter().copied().collect(),
        selected_edges: snapshot.selected_edges.iter().copied().collect(),
        selected_groups: snapshot.selected_groups.iter().copied().collect(),
    }
}
