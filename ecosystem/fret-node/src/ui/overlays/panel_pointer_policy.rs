use fret_ui::{UiHost, retained_bridge::*};

use crate::ui::retained_event_tail;

use super::panel_item_state::promote_pointer_target_to_keyboard_item;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PanelPressRelease<T> {
    pub(super) had_pressed: bool,
    pub(super) activate: Option<T>,
}

pub(super) fn begin_panel_press<H: UiHost, T: Copy>(
    cx: &mut EventCx<'_, H>,
    keyboard_active: &mut Option<T>,
    pressed: &mut Option<T>,
    pressed_target: Option<T>,
) {
    cx.request_focus(cx.node);
    cx.stop_propagation();
    promote_pointer_target_to_keyboard_item(keyboard_active, pressed_target);

    let Some(target) = pressed_target else {
        return;
    };

    *pressed = Some(target);
    cx.capture_pointer(cx.node);
    retained_event_tail::request_paint_repaint(cx);
}

pub(super) fn sync_panel_hover<T: Copy + Eq>(
    hovered: &mut Option<T>,
    next_hover: Option<T>,
) -> bool {
    if *hovered == next_hover {
        return false;
    }

    *hovered = next_hover;
    true
}

pub(super) fn release_panel_press<T: Copy + Eq>(
    pressed: &mut Option<T>,
    released_on: Option<T>,
) -> PanelPressRelease<T> {
    let pressed_action = pressed.take();
    let had_pressed = pressed_action.is_some();
    let activate = pressed_action.filter(|action| released_on == Some(*action));
    PanelPressRelease {
        had_pressed,
        activate,
    }
}

#[cfg(test)]
mod tests {
    use super::{PanelPressRelease, release_panel_press, sync_panel_hover};

    #[test]
    fn sync_panel_hover_only_reports_real_changes() {
        let mut hovered = None::<u8>;
        assert!(sync_panel_hover(&mut hovered, Some(2)));
        assert_eq!(hovered, Some(2));
        assert!(!sync_panel_hover(&mut hovered, Some(2)));
        assert!(sync_panel_hover(&mut hovered, None));
        assert_eq!(hovered, None);
    }

    #[test]
    fn release_panel_press_only_activates_on_matching_release_target() {
        let mut pressed = Some(3_u8);
        assert_eq!(
            release_panel_press(&mut pressed, Some(3)),
            PanelPressRelease {
                had_pressed: true,
                activate: Some(3)
            }
        );
        assert_eq!(pressed, None);

        let mut pressed = Some(4_u8);
        assert_eq!(
            release_panel_press(&mut pressed, Some(5)),
            PanelPressRelease {
                had_pressed: true,
                activate: None
            }
        );
        assert_eq!(pressed, None);

        let mut pressed = None::<u8>;
        assert_eq!(
            release_panel_press(&mut pressed, Some(1)),
            PanelPressRelease {
                had_pressed: false,
                activate: None
            }
        );
    }
}
