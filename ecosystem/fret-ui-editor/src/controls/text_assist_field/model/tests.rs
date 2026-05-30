use std::sync::Arc;

use super::{TextAssistFieldOptions, TextAssistFieldSurface};

#[test]
fn text_assist_field_defaults_to_unbuffered_field_policy() {
    let options = TextAssistFieldOptions::default();
    assert!(!options.field.buffered);
    assert!(matches!(options.surface, TextAssistFieldSurface::Inline));
    assert_eq!(options.list_label.as_ref(), "Suggestions");
    assert_eq!(options.empty_label.as_ref(), "No matches");
}

#[test]
fn text_assist_field_item_test_id_prefix_can_fallback_to_list_test_id() {
    let options = TextAssistFieldOptions {
        list_test_id: Some(Arc::from("editor.name-assist.list")),
        ..Default::default()
    };
    let prefix = options
        .item_test_id_prefix
        .clone()
        .or_else(|| options.list_test_id.clone());
    assert_eq!(prefix.as_deref(), Some("editor.name-assist.list"));
}
