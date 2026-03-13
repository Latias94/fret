use super::*;

#[test]
fn matches_delete_shortcut_uses_configured_binding() {
    assert!(matches_delete_shortcut(
        NodeGraphDeleteKey::BackspaceOrDelete,
        fret_core::KeyCode::Backspace,
    ));
    assert!(matches_delete_shortcut(
        NodeGraphDeleteKey::BackspaceOrDelete,
        fret_core::KeyCode::Delete,
    ));
    assert!(!matches_delete_shortcut(
        NodeGraphDeleteKey::Backspace,
        fret_core::KeyCode::Delete,
    ));
}
