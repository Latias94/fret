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
