use super::*;
use crate::interaction::NodeGraphModifierKey;

pub(super) fn should_ignore_key_down(focus_is_text_input: bool) -> bool {
    focus_is_text_input
}

pub(super) fn sync_keyboard_modifier_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    snapshot: &ViewSnapshot,
    modifiers: fret_core::Modifiers,
) {
    canvas.interaction.multi_selection_active =
        multi_selection_active(snapshot.interaction.multi_selection_key, modifiers);
}

fn multi_selection_active(key: NodeGraphModifierKey, modifiers: fret_core::Modifiers) -> bool {
    key.is_pressed(modifiers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_ignore_key_down_tracks_text_input_focus() {
        assert!(should_ignore_key_down(true));
        assert!(!should_ignore_key_down(false));
    }

    #[test]
    fn multi_selection_active_respects_modifier_key_policy() {
        assert!(multi_selection_active(
            NodeGraphModifierKey::None,
            fret_core::Modifiers::default()
        ));
        assert!(multi_selection_active(
            NodeGraphModifierKey::CtrlOrMeta,
            fret_core::Modifiers {
                ctrl: true,
                ..fret_core::Modifiers::default()
            }
        ));
        assert!(multi_selection_active(
            NodeGraphModifierKey::Alt,
            fret_core::Modifiers {
                alt_gr: true,
                ..fret_core::Modifiers::default()
            }
        ));
        assert!(!multi_selection_active(
            NodeGraphModifierKey::Shift,
            fret_core::Modifiers::default()
        ));
    }
}
