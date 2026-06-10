pub(super) fn assert_selection_commands_owner_split(selection_commands_source: &str) {
    for needle in [
        "mod delete;",
        "mod duplicate;",
        "pub(in super::super) use delete::{",
    ] {
        assert!(
            selection_commands_source.contains(needle),
            "the demo-local collection selection command hub should keep sub-owner re-exports explicit; missing `{needle}`"
        );
    }
}
