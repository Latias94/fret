use super::text_field_escape_clear_should_handle_key;
use fret_core::KeyCode;

#[test]
fn text_field_escape_clear_handles_escape_only() {
    assert!(text_field_escape_clear_should_handle_key(KeyCode::Escape));
    assert!(!text_field_escape_clear_should_handle_key(KeyCode::Enter));
}
