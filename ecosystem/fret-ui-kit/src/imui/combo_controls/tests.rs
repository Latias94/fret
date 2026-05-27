use super::trigger::combo_trigger_a11y_label;

#[test]
fn combo_trigger_a11y_label_formats_label_and_preview_inline() {
    assert_eq!(&*combo_trigger_a11y_label("Theme", "Dark"), "Theme: Dark");
}

#[test]
fn combo_trigger_a11y_label_uses_preview_only_when_label_is_empty() {
    assert_eq!(&*combo_trigger_a11y_label("", "Dark"), "Dark");
}
