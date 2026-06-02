use super::{EnumSelectTriggerKeyIntent, enum_select_trigger_key_intent};
use fret_core::KeyCode;

#[test]
fn enum_select_trigger_keys_open_on_activation_and_arrow_down() {
    for key in [
        KeyCode::Enter,
        KeyCode::NumpadEnter,
        KeyCode::Space,
        KeyCode::ArrowDown,
    ] {
        assert_eq!(
            enum_select_trigger_key_intent(key),
            EnumSelectTriggerKeyIntent::Open
        );
    }
}

#[test]
fn enum_select_trigger_keys_close_on_escape_only() {
    assert_eq!(
        enum_select_trigger_key_intent(KeyCode::Escape),
        EnumSelectTriggerKeyIntent::Close
    );

    assert_eq!(
        enum_select_trigger_key_intent(KeyCode::ArrowUp),
        EnumSelectTriggerKeyIntent::Ignore
    );
}
