use fret_core::{KeyCode, MouseButton};

use super::blackboard_policy::BlackboardAction;
use super::panel_item_state::{
    clear_panel_item_state, promote_pointer_target_to_keyboard_item, select_panel_keyboard_item,
};
use super::panel_navigation_policy::{PanelKeyboardAction, panel_keyboard_action};
use super::panel_pointer_policy::{PanelPressRelease, release_panel_press, sync_panel_hover};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BlackboardInteractionState {
    pub(super) hovered: Option<BlackboardAction>,
    pub(super) pressed: Option<BlackboardAction>,
    pub(super) keyboard_active: Option<BlackboardAction>,
}

impl BlackboardInteractionState {
    pub(super) fn new(
        hovered: Option<BlackboardAction>,
        pressed: Option<BlackboardAction>,
        keyboard_active: Option<BlackboardAction>,
    ) -> Self {
        Self {
            hovered,
            pressed,
            keyboard_active,
        }
    }

    pub(super) fn clear(&mut self) {
        clear_panel_item_state(
            &mut self.hovered,
            &mut self.pressed,
            &mut self.keyboard_active,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlackboardKeyboardInteractionPlan {
    Ignore,
    Select {
        action: BlackboardAction,
        finish_event: bool,
    },
    Activate {
        action: BlackboardAction,
        finish_event: bool,
    },
    FocusCanvas {
        finish_event: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BlackboardHoverInteractionPlan {
    pub(super) cursor_pointer: bool,
    pub(super) repaint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BlackboardPointerDownInteractionPlan {
    pub(super) request_focus: bool,
    pub(super) stop_propagation: bool,
    pub(super) capture_pointer: bool,
    pub(super) repaint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BlackboardPointerUpInteractionPlan {
    pub(super) stop_propagation: bool,
    pub(super) release_capture: bool,
    pub(super) repaint: bool,
    pub(super) activate: Option<BlackboardAction>,
}

pub(super) fn plan_blackboard_keyboard_interaction(
    state: &mut BlackboardInteractionState,
    key: KeyCode,
    items: &[BlackboardAction],
) -> BlackboardKeyboardInteractionPlan {
    match panel_keyboard_action(key, state.keyboard_active, items) {
        PanelKeyboardAction::Select(action) => {
            select_panel_keyboard_item(
                &mut state.hovered,
                &mut state.pressed,
                &mut state.keyboard_active,
                action,
            );
            BlackboardKeyboardInteractionPlan::Select {
                action,
                finish_event: true,
            }
        }
        PanelKeyboardAction::Activate(action) => BlackboardKeyboardInteractionPlan::Activate {
            action,
            finish_event: true,
        },
        PanelKeyboardAction::FocusCanvas => {
            state.clear();
            BlackboardKeyboardInteractionPlan::FocusCanvas { finish_event: true }
        }
        PanelKeyboardAction::Ignore => BlackboardKeyboardInteractionPlan::Ignore,
    }
}

pub(super) fn plan_blackboard_hover_interaction(
    state: &mut BlackboardInteractionState,
    hovered: Option<BlackboardAction>,
) -> BlackboardHoverInteractionPlan {
    BlackboardHoverInteractionPlan {
        cursor_pointer: hovered.is_some(),
        repaint: sync_panel_hover(&mut state.hovered, hovered),
    }
}

pub(super) fn plan_blackboard_pointer_down_interaction(
    state: &mut BlackboardInteractionState,
    button: MouseButton,
    panel_contains_pointer: bool,
    target: Option<BlackboardAction>,
) -> Option<BlackboardPointerDownInteractionPlan> {
    if button != MouseButton::Left || !panel_contains_pointer {
        return None;
    }

    promote_pointer_target_to_keyboard_item(&mut state.keyboard_active, target);
    if let Some(target) = target {
        state.pressed = Some(target);
    }

    Some(BlackboardPointerDownInteractionPlan {
        request_focus: true,
        stop_propagation: true,
        capture_pointer: target.is_some(),
        repaint: target.is_some(),
    })
}

pub(super) fn plan_blackboard_pointer_up_interaction(
    state: &mut BlackboardInteractionState,
    button: MouseButton,
    panel_contains_pointer: bool,
    released_on: Option<BlackboardAction>,
) -> Option<BlackboardPointerUpInteractionPlan> {
    if button != MouseButton::Left {
        return None;
    }

    let PanelPressRelease {
        had_pressed,
        activate,
    } = release_panel_press(&mut state.pressed, released_on);

    Some(BlackboardPointerUpInteractionPlan {
        stop_propagation: panel_contains_pointer,
        release_capture: true,
        repaint: had_pressed,
        activate,
    })
}

#[cfg(test)]
mod tests {
    use fret_core::{KeyCode, MouseButton};

    use crate::core::SymbolId;
    use crate::ui::overlays::blackboard_interaction_policy::{
        BlackboardInteractionState, BlackboardKeyboardInteractionPlan,
        plan_blackboard_hover_interaction, plan_blackboard_keyboard_interaction,
        plan_blackboard_pointer_down_interaction, plan_blackboard_pointer_up_interaction,
    };
    use crate::ui::overlays::blackboard_policy::BlackboardAction;

    fn symbol_action() -> BlackboardAction {
        BlackboardAction::Rename {
            symbol: SymbolId::from_u128(0),
        }
    }

    fn items() -> [BlackboardAction; 2] {
        [BlackboardAction::AddSymbol, symbol_action()]
    }

    #[test]
    fn blackboard_keyboard_interaction_selects_activates_focuses_and_ignores() {
        let mut state = BlackboardInteractionState::new(None, None, None);

        assert_eq!(
            plan_blackboard_keyboard_interaction(&mut state, KeyCode::ArrowDown, &items()),
            BlackboardKeyboardInteractionPlan::Select {
                action: symbol_action(),
                finish_event: true,
            }
        );
        assert_eq!(state.hovered, None);
        assert_eq!(state.pressed, None);
        assert_eq!(state.keyboard_active, Some(symbol_action()));

        assert_eq!(
            plan_blackboard_keyboard_interaction(&mut state, KeyCode::Enter, &items()),
            BlackboardKeyboardInteractionPlan::Activate {
                action: symbol_action(),
                finish_event: true,
            }
        );
        assert_eq!(state.keyboard_active, Some(symbol_action()));

        assert_eq!(
            plan_blackboard_keyboard_interaction(&mut state, KeyCode::Escape, &items()),
            BlackboardKeyboardInteractionPlan::FocusCanvas { finish_event: true }
        );
        assert_eq!(state, BlackboardInteractionState::new(None, None, None));

        assert_eq!(
            plan_blackboard_keyboard_interaction(&mut state, KeyCode::KeyA, &items()),
            BlackboardKeyboardInteractionPlan::Ignore
        );
    }

    #[test]
    fn blackboard_hover_interaction_updates_hover_and_repaint_only_on_change() {
        let mut state = BlackboardInteractionState::new(None, None, None);
        let plan = plan_blackboard_hover_interaction(&mut state, Some(symbol_action()));
        assert!(plan.cursor_pointer);
        assert!(plan.repaint);
        assert_eq!(state.hovered, Some(symbol_action()));

        let plan = plan_blackboard_hover_interaction(&mut state, Some(symbol_action()));
        assert!(plan.cursor_pointer);
        assert!(!plan.repaint);

        let plan = plan_blackboard_hover_interaction(&mut state, None);
        assert!(!plan.cursor_pointer);
        assert!(plan.repaint);
        assert_eq!(state.hovered, None);
    }

    #[test]
    fn blackboard_pointer_down_promotes_keyboard_and_captures_only_panel_actions() {
        let mut state = BlackboardInteractionState::new(None, None, None);
        let plan = plan_blackboard_pointer_down_interaction(
            &mut state,
            MouseButton::Left,
            true,
            Some(symbol_action()),
        )
        .expect("left pointer down inside panel");

        assert!(plan.request_focus);
        assert!(plan.stop_propagation);
        assert!(plan.capture_pointer);
        assert!(plan.repaint);
        assert_eq!(state.keyboard_active, Some(symbol_action()));
        assert_eq!(state.pressed, Some(symbol_action()));

        let mut state =
            BlackboardInteractionState::new(None, None, Some(BlackboardAction::AddSymbol));
        let plan =
            plan_blackboard_pointer_down_interaction(&mut state, MouseButton::Left, true, None)
                .expect("left pointer down inside panel but outside action");
        assert!(plan.request_focus);
        assert!(plan.stop_propagation);
        assert!(!plan.capture_pointer);
        assert!(!plan.repaint);
        assert_eq!(state.keyboard_active, Some(BlackboardAction::AddSymbol));
        assert_eq!(state.pressed, None);

        assert!(
            plan_blackboard_pointer_down_interaction(
                &mut state,
                MouseButton::Left,
                false,
                Some(symbol_action()),
            )
            .is_none()
        );
        assert!(
            plan_blackboard_pointer_down_interaction(
                &mut state,
                MouseButton::Right,
                true,
                Some(symbol_action()),
            )
            .is_none()
        );
    }

    #[test]
    fn blackboard_pointer_up_releases_capture_repaints_and_activates_matching_press() {
        let mut state = BlackboardInteractionState::new(
            Some(symbol_action()),
            Some(symbol_action()),
            Some(symbol_action()),
        );
        let plan = plan_blackboard_pointer_up_interaction(
            &mut state,
            MouseButton::Left,
            true,
            Some(symbol_action()),
        )
        .expect("left pointer up");

        assert!(plan.stop_propagation);
        assert!(plan.release_capture);
        assert!(plan.repaint);
        assert_eq!(plan.activate, Some(symbol_action()));
        assert_eq!(state.pressed, None);

        let mut state =
            BlackboardInteractionState::new(None, Some(BlackboardAction::AddSymbol), None);
        let plan = plan_blackboard_pointer_up_interaction(
            &mut state,
            MouseButton::Left,
            false,
            Some(symbol_action()),
        )
        .expect("left pointer up outside panel");
        assert!(!plan.stop_propagation);
        assert!(plan.release_capture);
        assert!(plan.repaint);
        assert_eq!(plan.activate, None);
        assert_eq!(state.pressed, None);

        assert!(
            plan_blackboard_pointer_up_interaction(
                &mut state,
                MouseButton::Right,
                true,
                Some(symbol_action()),
            )
            .is_none()
        );
    }
}
