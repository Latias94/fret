#![cfg(feature = "imui")]

use fret_core::SemanticsRole;
use fret_ui_kit::imui::{InputTextMode, InputTextOptions};

#[test]
fn input_text_option_defaults_compile() {
    let options = InputTextOptions::default();
    assert!(options.enabled);
    assert!(options.focusable);
    assert!(!options.read_only);
    assert!(!options.select_all_on_focus);
    assert_eq!(options.mode, InputTextMode::PlainText);
    assert!(options.filters.is_empty());
    assert!(options.custom_filter.is_none());
    assert!(options.a11y_label.is_none());
    assert_eq!(options.a11y_role, Some(SemanticsRole::TextField));
    assert!(options.placeholder.is_none());
    assert!(options.test_id.is_none());
    assert!(options.submit_command.is_none());
    assert!(options.cancel_command.is_none());
    assert!(options.completion_command.is_none());
    assert!(options.history_previous_command.is_none());
    assert!(options.history_next_command.is_none());
    assert!(options.undo_command.is_none());
    assert!(options.redo_command.is_none());
    assert!(!options.completion_command_repeat);
    assert!(!options.history_command_repeat);
    assert!(!options.undo_redo_command_repeat);
}

#[test]
fn input_text_mode_password_compiles() {
    let options = InputTextOptions {
        mode: InputTextMode::Password,
        ..Default::default()
    };
    assert_eq!(options.mode, InputTextMode::Password);
}
