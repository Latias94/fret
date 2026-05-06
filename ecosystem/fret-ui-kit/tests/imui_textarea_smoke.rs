#![cfg(feature = "imui")]

use fret_runtime::CommandId;
use fret_ui_kit::imui::{TextAreaOptions, TextAreaSubmitKey};

#[test]
fn textarea_command_policy_options_compile() {
    let options = TextAreaOptions {
        submit_command: Some(CommandId::from("editor.submit")),
        cancel_command: Some(CommandId::from("editor.cancel")),
        submit_key: TextAreaSubmitKey::Enter,
        submit_cancel_command_repeat: true,
        ..Default::default()
    };

    assert!(options.submit_command.is_some());
    assert!(options.cancel_command.is_some());
    assert_eq!(options.submit_key, TextAreaSubmitKey::Enter);
    assert!(options.submit_cancel_command_repeat);
}
