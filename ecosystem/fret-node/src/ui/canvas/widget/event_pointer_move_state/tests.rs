use super::*;

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
