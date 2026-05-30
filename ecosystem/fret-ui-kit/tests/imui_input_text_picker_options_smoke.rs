#![cfg(feature = "imui")]

use fret_core::{Px, Size};
use fret_ui_kit::imui::{InputTextPickerFilter, InputTextPickerOptions};

#[test]
fn input_text_picker_filter_matches_compile() {
    assert!(InputTextPickerFilter::ContainsCaseInsensitive.matches("cam", "Scene Camera"));
    assert!(InputTextPickerFilter::PrefixCaseInsensitive.matches("cam", "Camera"));
    assert!(!InputTextPickerFilter::PrefixCaseInsensitive.matches("cam", "Scene Camera"));
    assert!(InputTextPickerFilter::None.matches("anything", "Scene Camera"));
    assert!(InputTextPickerFilter::ContainsCaseInsensitive.matches("", "Scene Camera"));
}

#[test]
fn input_text_picker_option_defaults_compile() {
    let options = InputTextPickerOptions::default();
    assert_eq!(
        options.filter,
        InputTextPickerFilter::ContainsCaseInsensitive
    );
    assert_eq!(options.max_items, 8);
    assert!(options.open_on_focus);
    assert!(!options.open_when_empty);
    assert!(options.hide_when_exact_match);
    assert!(options.keyboard_navigation);
    assert!(!options.keyboard_repeat);
    assert!(options.test_id.is_none());
    assert!(!options.popup.modal);
    assert!(!options.popup.auto_focus);
    assert_eq!(
        options.popup.estimated_size,
        Size::new(Px(220.0), Px(160.0))
    );
}
