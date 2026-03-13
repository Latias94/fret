use super::super::*;
use crate::ui::canvas::state::InteractionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct EdgeAnchorInteractionState {
    pub(super) hovered: bool,
    pub(super) active: bool,
}

pub(super) fn edge_anchor_interaction_state(
    interaction: &InteractionState,
    target_edge_id: Option<EdgeId>,
    endpoint: EdgeEndpoint,
) -> EdgeAnchorInteractionState {
    let hovered = interaction
        .hover_edge_anchor
        .is_some_and(|(edge, ep)| Some(edge) == target_edge_id && ep == endpoint);
    let active = interaction
        .wire_drag
        .as_ref()
        .is_some_and(|drag| match &drag.kind {
            WireDragKind::Reconnect {
                edge, endpoint: ep, ..
            } => Some(*edge) == target_edge_id && *ep == endpoint,
            _ => false,
        });

    EdgeAnchorInteractionState { hovered, active }
}

#[cfg(test)]
mod tests;
