#![cfg(feature = "imui")]

use fret_core::SemanticsRole;
use fret_ui_kit::imui::{SelectableOptions, SliderOptions, TabItemOptions};

#[test]
fn selectable_option_defaults_compile() {
    let options = SelectableOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(!options.selected);
    assert!(!options.highlighted);
    assert!(options.close_popup.is_none());
    assert!(options.a11y_label.is_none());
    assert_eq!(options.a11y_role, Some(SemanticsRole::ListBoxOption));
    assert!(options.test_id.is_none());
    assert!(options.activate_shortcut.is_none());
    assert!(!options.shortcut_repeat);
}

#[test]
fn tab_item_option_defaults_compile() {
    let options = TabItemOptions::default();
    assert!(options.enabled);
    assert!(!options.default_selected);
    assert!(options.test_id.is_none());
    assert!(options.panel_test_id.is_none());
    assert!(options.activate_shortcut.is_none());
    assert!(!options.shortcut_repeat);
}

#[test]
fn slider_option_defaults_compile() {
    let options = SliderOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(options.a11y_label.is_none());
    assert!(options.test_id.is_none());
    assert_eq!(options.min, 0.0);
    assert_eq!(options.max, 100.0);
    assert_eq!(options.step, 1.0);
}
