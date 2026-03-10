use crate::ui::canvas::state::InteractionState;

pub(in super::super) fn clear_focused_port_hints(interaction: &mut InteractionState) {
    interaction.focused_port_valid = false;
    interaction.focused_port_convertible = false;
}

pub(in super::super) fn clear_hover_port_hints(interaction: &mut InteractionState) {
    interaction.hover_port = None;
    interaction.hover_port_valid = false;
    interaction.hover_port_convertible = false;
    interaction.hover_port_diagnostic = None;
}

pub(in super::super) fn clear_edge_focus(interaction: &mut InteractionState) {
    interaction.focused_edge = None;
}

pub(in super::super) fn clear_edge_focus_and_hover_port_hints(interaction: &mut InteractionState) {
    clear_edge_focus(interaction);
    clear_hover_port_hints(interaction);
}

pub(in super::super) fn clear_hover_edge_focus_and_hover_port_hints(
    interaction: &mut InteractionState,
) {
    interaction.hover_edge = None;
    clear_edge_focus_and_hover_port_hints(interaction);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{EdgeId, PortId};
    use crate::rules::DiagnosticSeverity;

    #[test]
    fn clear_hover_edge_focus_and_hover_port_hints_clears_edge_related_state() {
        let mut interaction = InteractionState {
            focused_edge: Some(EdgeId::from_u128(3)),
            hover_edge: Some(EdgeId::from_u128(4)),
            hover_port: Some(PortId::from_u128(5)),
            hover_port_valid: true,
            hover_port_convertible: true,
            hover_port_diagnostic: Some((DiagnosticSeverity::Error, "diag".into())),
            ..Default::default()
        };

        clear_hover_edge_focus_and_hover_port_hints(&mut interaction);

        assert_eq!(interaction.focused_edge, None);
        assert_eq!(interaction.hover_edge, None);
        assert_eq!(interaction.hover_port, None);
        assert!(!interaction.hover_port_valid);
        assert!(!interaction.hover_port_convertible);
        assert_eq!(interaction.hover_port_diagnostic, None);
    }
}
