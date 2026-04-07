use crate::ui::canvas::widget::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::ui::canvas::widget) enum ContextMenuHoverEdgePolicy {
    Preserve,
    Clear,
}

pub(super) fn apply_context_menu_open_state(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
    hover_edge_policy: ContextMenuHoverEdgePolicy,
) {
    restore_context_menu(interaction, menu);
    if matches!(hover_edge_policy, ContextMenuHoverEdgePolicy::Clear) {
        interaction.hover_edge = None;
    }
}

pub(super) fn clear_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    interaction.context_menu.take().is_some()
}

pub(super) fn take_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> Option<ContextMenuState> {
    interaction.context_menu.take()
}

pub(super) fn restore_context_menu(
    interaction: &mut crate::ui::canvas::state::InteractionState,
    menu: ContextMenuState,
) {
    interaction.context_menu = Some(menu);
}

#[cfg(test)]
mod tests {
    use super::{ContextMenuHoverEdgePolicy, apply_context_menu_open_state};
    use crate::core::EdgeId;
    use crate::ui::canvas::state::{ContextMenuState, ContextMenuTarget, InteractionState};
    use crate::ui::presenter::{NodeGraphContextMenuAction, NodeGraphContextMenuItem};
    use fret_core::{Point, Px};
    use std::sync::Arc;

    fn test_menu(target: ContextMenuTarget) -> ContextMenuState {
        ContextMenuState {
            origin: Point::new(Px(10.0), Px(20.0)),
            invoked_at: Point::new(Px(10.0), Px(20.0)),
            target,
            items: vec![NodeGraphContextMenuItem {
                label: Arc::<str>::from("Menu Item"),
                enabled: true,
                action: NodeGraphContextMenuAction::Custom(1),
            }],
            candidates: Vec::new(),
            hovered_item: None,
            active_item: 0,
            typeahead: String::new(),
        }
    }

    #[test]
    fn open_state_installs_context_menu_and_clears_hover_edge_when_requested() {
        let hovered_edge = EdgeId::new();
        let menu = test_menu(ContextMenuTarget::Background);
        let mut interaction = InteractionState {
            hover_edge: Some(hovered_edge),
            ..Default::default()
        };

        apply_context_menu_open_state(
            &mut interaction,
            menu.clone(),
            ContextMenuHoverEdgePolicy::Clear,
        );

        assert!(interaction.hover_edge.is_none());
        assert!(matches!(
            interaction.context_menu,
            Some(ContextMenuState {
                target: ContextMenuTarget::Background,
                ..
            })
        ));
    }

    #[test]
    fn open_state_preserves_hover_edge_when_cleanup_is_disabled() {
        let hovered_edge = EdgeId::new();
        let menu = test_menu(ContextMenuTarget::Background);
        let mut interaction = InteractionState {
            hover_edge: Some(hovered_edge),
            ..Default::default()
        };

        apply_context_menu_open_state(&mut interaction, menu, ContextMenuHoverEdgePolicy::Preserve);

        assert_eq!(interaction.hover_edge, Some(hovered_edge));
        assert!(interaction.context_menu.is_some());
    }
}
