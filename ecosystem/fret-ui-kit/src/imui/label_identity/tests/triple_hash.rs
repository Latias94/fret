use super::*;

#[test]
fn imui_label_identity_uses_triple_hash_suffix_as_stable_identity() {
    let parts = parse_label_identity("Compiling 42%###build-progress");
    assert_eq!(parts.visible, "Compiling 42%");
    assert_eq!(parts.identity, "build-progress");
}

#[test]
fn imui_label_identity_supports_hidden_label_with_triple_hash_id() {
    let parts = parse_label_identity("###hidden-stable");
    assert_eq!(parts.visible, "");
    assert_eq!(parts.identity, "hidden-stable");
}

#[test]
fn imui_label_identity_triple_hash_takes_identity_precedence() {
    let parts = parse_label_identity("Play##toolbar###stable-play");
    assert_eq!(parts.visible, "Play");
    assert_eq!(parts.identity, "stable-play");
}
