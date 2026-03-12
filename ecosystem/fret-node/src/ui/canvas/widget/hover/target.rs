use crate::core::EdgeId;
use crate::ui::canvas::state::{InteractionState, ViewSnapshot};

pub(in super::super) fn hover_anchor_target_edge(
    interaction: &InteractionState,
    snapshot: &ViewSnapshot,
) -> Option<EdgeId> {
    interaction
        .focused_edge
        .or_else(|| (snapshot.selected_edges.len() == 1).then(|| snapshot.selected_edges[0]))
}

#[cfg(test)]
mod tests;
