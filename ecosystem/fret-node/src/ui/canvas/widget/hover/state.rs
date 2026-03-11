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
mod tests {
    use super::*;
    use crate::core::EdgeId;
    use crate::rules::EdgeEndpoint;

    #[test]
    fn sync_hover_edge_state_reports_and_applies_changes() {
        let edge = EdgeId::from_u128(1);
        let other_edge = EdgeId::from_u128(2);
        let mut interaction = InteractionState::default();

        let (anchor_changed, edge_changed) = sync_hover_edge_state(
            &mut interaction,
            Some((edge, EdgeEndpoint::From)),
            Some(other_edge),
        );

        assert!(anchor_changed);
        assert!(edge_changed);
        assert_eq!(
            interaction.hover_edge_anchor,
            Some((edge, EdgeEndpoint::From))
        );
        assert_eq!(interaction.hover_edge, Some(other_edge));
    }

    #[test]
    fn sync_hover_edge_state_is_noop_when_state_matches() {
        let edge = EdgeId::from_u128(1);
        let mut interaction = InteractionState::default();
        interaction.hover_edge_anchor = Some((edge, EdgeEndpoint::To));
        interaction.hover_edge = Some(edge);

        let (anchor_changed, edge_changed) =
            sync_hover_edge_state(&mut interaction, Some((edge, EdgeEndpoint::To)), Some(edge));

        assert!(!anchor_changed);
        assert!(!edge_changed);
    }
}
