use super::*;

#[test]
fn apply_searcher_query_key_handles_ascii_and_backspace() {
    let mut query = String::from("ab");

    assert!(apply_searcher_query_key(
        &mut query,
        KeyCode::Backspace,
        Modifiers::default(),
    ));
    assert_eq!(query, "a");

    assert!(apply_searcher_query_key(
        &mut query,
        KeyCode::KeyC,
        Modifiers::default(),
    ));
    assert_eq!(query, "ac");

    assert!(!apply_searcher_query_key(
        &mut query,
        KeyCode::KeyV,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
    ));
    assert_eq!(query, "ac");
}
