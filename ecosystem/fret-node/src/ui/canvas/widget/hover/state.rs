use crate::core::EdgeId;
use crate::rules::EdgeEndpoint;
use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn sync_hover_edge_state(
    interaction: &mut InteractionState,
    new_hover_anchor: Option<(EdgeId, EdgeEndpoint)>,
    new_hover: Option<EdgeId>,
) -> (bool, bool) {
    let anchor_changed = interaction.hover_edge_anchor != new_hover_anchor;
    if anchor_changed {
        interaction.hover_edge_anchor = new_hover_anchor;
    }

    let edge_changed = interaction.hover_edge != new_hover;
    if edge_changed {
        interaction.hover_edge = new_hover;
    }

    (anchor_changed, edge_changed)
}

#[cfg(test)]
mod tests;
