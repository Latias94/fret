#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PanelItemVisualState {
    pub(super) hovered: bool,
    pub(super) pressed: bool,
    pub(super) keyboard: bool,
}

impl PanelItemVisualState {
    pub(super) fn active(self) -> bool {
        self.hovered || self.pressed || self.keyboard
    }
}

pub(super) fn select_panel_keyboard_item<T: Copy>(
    hovered: &mut Option<T>,
    pressed: &mut Option<T>,
    keyboard_active: &mut Option<T>,
    item: T,
) {
    *hovered = None;
    *pressed = None;
    *keyboard_active = Some(item);
}

pub(super) fn clear_panel_item_state<T>(
    hovered: &mut Option<T>,
    pressed: &mut Option<T>,
    keyboard_active: &mut Option<T>,
) {
    *hovered = None;
    *pressed = None;
    *keyboard_active = None;
}

pub(super) fn promote_pointer_target_to_keyboard_item<T: Copy>(
    keyboard_active: &mut Option<T>,
    pointer_target: Option<T>,
) {
    if let Some(target) = pointer_target {
        *keyboard_active = Some(target);
    }
}

pub(super) fn panel_item_visual_state<T: Copy + Eq>(
    item: T,
    hovered: Option<T>,
    pressed: Option<T>,
    keyboard_active: Option<T>,
    keyboard_visible: bool,
    keyboard_requires_pointer_idle: bool,
) -> PanelItemVisualState {
    let hovered_match = hovered == Some(item);
    let pressed_match = pressed == Some(item);
    let keyboard_match = keyboard_active == Some(item)
        && keyboard_visible
        && (!keyboard_requires_pointer_idle || (hovered.is_none() && pressed.is_none()));
    PanelItemVisualState {
        hovered: hovered_match,
        pressed: pressed_match,
        keyboard: keyboard_match,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PanelItemVisualState, clear_panel_item_state, panel_item_visual_state,
        promote_pointer_target_to_keyboard_item, select_panel_keyboard_item,
    };

    #[test]
    fn select_panel_keyboard_item_resets_pointer_slots() {
        let mut hovered = Some(1_u8);
        let mut pressed = Some(2_u8);
        let mut keyboard = None;
        select_panel_keyboard_item(&mut hovered, &mut pressed, &mut keyboard, 3_u8);
        assert_eq!(hovered, None);
        assert_eq!(pressed, None);
        assert_eq!(keyboard, Some(3));
    }

    #[test]
    fn clear_panel_item_state_drops_all_slots() {
        let mut hovered = Some(1_u8);
        let mut pressed = Some(2_u8);
        let mut keyboard = Some(3_u8);
        clear_panel_item_state(&mut hovered, &mut pressed, &mut keyboard);
        assert_eq!(hovered, None);
        assert_eq!(pressed, None);
        assert_eq!(keyboard, None);
    }

    #[test]
    fn pointer_target_promotion_preserves_existing_keyboard_item_when_none() {
        let mut keyboard = Some(5_u8);
        promote_pointer_target_to_keyboard_item(&mut keyboard, None::<u8>);
        assert_eq!(keyboard, Some(5));
        promote_pointer_target_to_keyboard_item(&mut keyboard, Some(7_u8));
        assert_eq!(keyboard, Some(7));
    }

    #[test]
    fn panel_item_visual_state_can_require_pointer_idle_for_keyboard_highlight() {
        assert_eq!(
            panel_item_visual_state(3_u8, None, None, Some(3), true, true),
            PanelItemVisualState {
                hovered: false,
                pressed: false,
                keyboard: true
            }
        );
        assert_eq!(
            panel_item_visual_state(3_u8, Some(3), None, Some(3), true, true),
            PanelItemVisualState {
                hovered: true,
                pressed: false,
                keyboard: false
            }
        );
    }

    #[test]
    fn panel_item_visual_state_can_keep_keyboard_highlight_visible_without_focus_gate() {
        let state = panel_item_visual_state(4_u8, Some(2), None, Some(4), true, false);
        assert!(state.keyboard);
        assert!(state.active());
    }
}
