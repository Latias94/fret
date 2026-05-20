use fret_core::{KeyCode, MouseButton};

use super::controls_host_policy::plan_controls_pointer_down_host;
use super::controls_policy::{ControlsButton, controls_buttons};
use super::panel_item_state::{
    clear_panel_item_state, promote_pointer_target_to_keyboard_item, select_panel_keyboard_item,
};
use super::panel_navigation_policy::{PanelKeyboardAction, panel_keyboard_action};
use super::panel_pointer_policy::{PanelPressRelease, release_panel_press, sync_panel_hover};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ControlsInteractionState {
    pub(super) hovered: Option<ControlsButton>,
    pub(super) pressed: Option<ControlsButton>,
    pub(super) keyboard_active: Option<ControlsButton>,
}

impl ControlsInteractionState {
    pub(super) fn new(
        hovered: Option<ControlsButton>,
        pressed: Option<ControlsButton>,
        keyboard_active: Option<ControlsButton>,
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
pub(super) enum ControlsKeyboardInteractionPlan {
    Ignore,
    Select {
        button: ControlsButton,
        finish_event: bool,
    },
    Activate {
        button: ControlsButton,
        finish_event: bool,
    },
    FocusCanvas {
        finish_event: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ControlsHoverInteractionPlan {
    pub(super) cursor_pointer: bool,
    pub(super) repaint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ControlsPointerDownInteractionPlan {
    pub(super) request_focus: bool,
    pub(super) stop_propagation: bool,
    pub(super) capture_pointer: bool,
    pub(super) repaint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ControlsPointerUpInteractionPlan {
    pub(super) release_capture: bool,
    pub(super) finish_event: bool,
    pub(super) activate: Option<ControlsButton>,
}

pub(super) fn plan_controls_keyboard_interaction(
    state: &mut ControlsInteractionState,
    key: KeyCode,
) -> ControlsKeyboardInteractionPlan {
    match panel_keyboard_action(key, state.keyboard_active, controls_buttons()) {
        PanelKeyboardAction::Select(button) => {
            select_panel_keyboard_item(
                &mut state.hovered,
                &mut state.pressed,
                &mut state.keyboard_active,
                button,
            );
            ControlsKeyboardInteractionPlan::Select {
                button,
                finish_event: true,
            }
        }
        PanelKeyboardAction::Activate(button) => {
            state.clear();
            ControlsKeyboardInteractionPlan::Activate {
                button,
                finish_event: true,
            }
        }
        PanelKeyboardAction::FocusCanvas => {
            state.clear();
            ControlsKeyboardInteractionPlan::FocusCanvas { finish_event: true }
        }
        PanelKeyboardAction::Ignore => ControlsKeyboardInteractionPlan::Ignore,
    }
}

pub(super) fn plan_controls_hover_interaction(
    state: &mut ControlsInteractionState,
    hovered: Option<ControlsButton>,
) -> ControlsHoverInteractionPlan {
    ControlsHoverInteractionPlan {
        cursor_pointer: hovered.is_some(),
        repaint: sync_panel_hover(&mut state.hovered, hovered),
    }
}

pub(super) fn plan_controls_pointer_down_interaction(
    state: &mut ControlsInteractionState,
    button: MouseButton,
    target: Option<ControlsButton>,
) -> Option<ControlsPointerDownInteractionPlan> {
    if button != MouseButton::Left {
        return None;
    }

    promote_pointer_target_to_keyboard_item(&mut state.keyboard_active, target);
    if let Some(target) = target {
        state.pressed = Some(target);
    }

    let host_plan = plan_controls_pointer_down_host(button, target)?;
    Some(ControlsPointerDownInteractionPlan {
        request_focus: host_plan.request_focus,
        stop_propagation: host_plan.stop_propagation,
        capture_pointer: host_plan.capture_pointer,
        repaint: host_plan.repaint,
    })
}

pub(super) fn plan_controls_pointer_up_interaction(
    state: &mut ControlsInteractionState,
    button: MouseButton,
    released_on: Option<ControlsButton>,
) -> Option<ControlsPointerUpInteractionPlan> {
    if button != MouseButton::Left {
        return None;
    }

    let PanelPressRelease {
        had_pressed,
        activate,
    } = release_panel_press(&mut state.pressed, released_on);

    if activate.is_some() {
        state.clear();
    }

    Some(ControlsPointerUpInteractionPlan {
        release_capture: true,
        finish_event: had_pressed,
        activate,
    })
}

#[cfg(test)]
mod tests {
    use fret_core::{KeyCode, MouseButton};

    use crate::ui::overlays::controls_interaction_policy::{
        ControlsInteractionState, ControlsKeyboardInteractionPlan, plan_controls_hover_interaction,
        plan_controls_keyboard_interaction, plan_controls_pointer_down_interaction,
        plan_controls_pointer_up_interaction,
    };
    use crate::ui::overlays::controls_policy::ControlsButton;

    #[test]
    fn controls_keyboard_interaction_selects_activates_focuses_and_ignores() {
        let mut state = ControlsInteractionState::new(None, None, None);

        assert_eq!(
            plan_controls_keyboard_interaction(&mut state, KeyCode::ArrowDown),
            ControlsKeyboardInteractionPlan::Select {
                button: ControlsButton::ZoomIn,
                finish_event: true,
            }
        );
        assert_eq!(state.hovered, None);
        assert_eq!(state.pressed, None);
        assert_eq!(state.keyboard_active, Some(ControlsButton::ZoomIn));

        assert_eq!(
            plan_controls_keyboard_interaction(&mut state, KeyCode::Enter),
            ControlsKeyboardInteractionPlan::Activate {
                button: ControlsButton::ZoomIn,
                finish_event: true,
            }
        );
        assert_eq!(state, ControlsInteractionState::new(None, None, None));

        assert_eq!(
            plan_controls_keyboard_interaction(&mut state, KeyCode::Escape),
            ControlsKeyboardInteractionPlan::FocusCanvas { finish_event: true }
        );
        assert_eq!(
            plan_controls_keyboard_interaction(&mut state, KeyCode::KeyA),
            ControlsKeyboardInteractionPlan::Ignore
        );
    }

    #[test]
    fn controls_hover_interaction_updates_hover_and_repaint_only_on_change() {
        let mut state = ControlsInteractionState::new(None, None, None);
        let plan = plan_controls_hover_interaction(&mut state, Some(ControlsButton::ZoomIn));
        assert!(plan.cursor_pointer);
        assert!(plan.repaint);
        assert_eq!(state.hovered, Some(ControlsButton::ZoomIn));

        let plan = plan_controls_hover_interaction(&mut state, Some(ControlsButton::ZoomIn));
        assert!(plan.cursor_pointer);
        assert!(!plan.repaint);

        let plan = plan_controls_hover_interaction(&mut state, None);
        assert!(!plan.cursor_pointer);
        assert!(plan.repaint);
        assert_eq!(state.hovered, None);
    }

    #[test]
    fn controls_pointer_down_promotes_keyboard_and_captures_only_buttons() {
        let mut state = ControlsInteractionState::new(None, None, None);
        let plan = plan_controls_pointer_down_interaction(
            &mut state,
            MouseButton::Left,
            Some(ControlsButton::FrameAll),
        )
        .expect("left pointer down");

        assert!(plan.request_focus);
        assert!(plan.stop_propagation);
        assert!(plan.capture_pointer);
        assert!(plan.repaint);
        assert_eq!(state.keyboard_active, Some(ControlsButton::FrameAll));
        assert_eq!(state.pressed, Some(ControlsButton::FrameAll));

        let mut state = ControlsInteractionState::new(None, None, Some(ControlsButton::ZoomIn));
        let plan = plan_controls_pointer_down_interaction(&mut state, MouseButton::Left, None)
            .expect("left pointer down inside panel but outside button");
        assert!(plan.request_focus);
        assert!(plan.stop_propagation);
        assert!(!plan.capture_pointer);
        assert!(!plan.repaint);
        assert_eq!(state.keyboard_active, Some(ControlsButton::ZoomIn));
        assert_eq!(state.pressed, None);

        assert!(
            plan_controls_pointer_down_interaction(
                &mut state,
                MouseButton::Right,
                Some(ControlsButton::ZoomIn),
            )
            .is_none()
        );
    }

    #[test]
    fn controls_pointer_up_releases_capture_finishes_and_activates_matching_press() {
        let mut state = ControlsInteractionState::new(
            Some(ControlsButton::ZoomIn),
            Some(ControlsButton::ZoomIn),
            Some(ControlsButton::ZoomIn),
        );
        let plan = plan_controls_pointer_up_interaction(
            &mut state,
            MouseButton::Left,
            Some(ControlsButton::ZoomIn),
        )
        .expect("left pointer up");

        assert!(plan.release_capture);
        assert!(plan.finish_event);
        assert_eq!(plan.activate, Some(ControlsButton::ZoomIn));
        assert_eq!(state, ControlsInteractionState::new(None, None, None));

        let mut state = ControlsInteractionState::new(None, Some(ControlsButton::ZoomOut), None);
        let plan = plan_controls_pointer_up_interaction(
            &mut state,
            MouseButton::Left,
            Some(ControlsButton::ZoomIn),
        )
        .expect("left pointer up");
        assert!(plan.release_capture);
        assert!(plan.finish_event);
        assert_eq!(plan.activate, None);
        assert_eq!(state.pressed, None);

        assert!(
            plan_controls_pointer_up_interaction(
                &mut state,
                MouseButton::Right,
                Some(ControlsButton::ZoomIn),
            )
            .is_none()
        );
    }
}
