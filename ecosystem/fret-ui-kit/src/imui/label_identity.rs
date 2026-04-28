#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ImUiLabelParts<'a> {
    pub visible: &'a str,
    pub identity: &'a str,
}

pub(super) fn parse_label_identity(label: &str) -> ImUiLabelParts<'_> {
    let visible_end = label.find("##").unwrap_or(label.len());
    let visible = &label[..visible_end];
    let identity = label
        .find("###")
        .map(|marker| &label[marker + 3..])
        .unwrap_or(label);

    ImUiLabelParts { visible, identity }
}

#[cfg(test)]
mod tests {
    use super::parse_label_identity;

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
}
