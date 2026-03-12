use std::sync::Arc;

use fret_core::NodeId;
use fret_runtime::{CommandId, Model};

use fret_ui_editor::controls::{
    TextField, TextFieldAssistiveSemantics, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome,
};

#[allow(dead_code)]
fn text_field_accepts_editor_text_extensions(value_model: &Model<String>) {
    let _field = TextField::new(value_model.clone())
        .on_outcome(Some(Arc::new(
            |_host, _action_cx, _outcome: TextFieldOutcome| {},
        )))
        .options(TextFieldOptions {
            id_source: Some(Arc::from("tests.text_field.password")),
            mode: TextFieldMode::Password,
            submit_command: Some(CommandId::from("tests.text_field.submit")),
            assistive_semantics: TextFieldAssistiveSemantics {
                active_descendant: Some(NodeId::default()),
                controls_element: Some(42),
                expanded: Some(true),
            },
            ..Default::default()
        });
}

#[test]
fn text_field_option_defaults_match_buffered_plain_text_baseline() {
    let options = TextFieldOptions::default();
    assert!(options.buffered);
    assert_eq!(options.blur_behavior, TextFieldBlurBehavior::Commit);
    assert_eq!(options.mode, TextFieldMode::PlainText);
    assert!(options.submit_command.is_none());
    assert_eq!(
        options.assistive_semantics,
        TextFieldAssistiveSemantics::default()
    );
}
