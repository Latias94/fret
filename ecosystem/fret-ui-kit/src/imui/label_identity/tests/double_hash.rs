use super::*;

#[test]
fn imui_label_identity_keeps_plain_label_visible_and_identifying() {
    let parts = parse_label_identity("Play");
    assert_eq!(parts.visible, "Play");
    assert_eq!(parts.identity, "Play");
}

#[test]
fn imui_label_identity_hides_double_hash_suffix_from_visible_label() {
    let parts = parse_label_identity("Play##toolbar");
    assert_eq!(parts.visible, "Play");
    assert_eq!(parts.identity, "Play##toolbar");
}

#[test]
fn imui_label_identity_supports_hidden_label_with_double_hash_id() {
    let parts = parse_label_identity("##toolbar-play");
    assert_eq!(parts.visible, "");
    assert_eq!(parts.identity, "##toolbar-play");
}
