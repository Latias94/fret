use std::sync::Arc;

use fret_core::NodeId;
use fret_runtime::{CommandId, Model};

use fret_ui_editor::controls::{
    TextField, TextFieldAssistiveSemantics, TextFieldBlurBehavior, TextFieldDraftController,
    TextFieldMode, TextFieldOptions, TextFieldOutcome,
};

#[allow(dead_code)]
fn text_field_accepts_editor_text_extensions(value_model: &Model<String>) {
    let draft_controller = TextFieldDraftController::new();
    let _field = TextField::new(value_model.clone())
        .on_outcome(Some(Arc::new(
            |_host, _action_cx, _outcome: TextFieldOutcome| {},
        )))
        .options(TextFieldOptions {
            id_source: Some(Arc::from("tests.text_field.password")),
            mode: TextFieldMode::Password,
            submit_command: Some(CommandId::from("tests.text_field.submit")),
            draft_controller: Some(draft_controller.clone()),
            assistive_semantics: TextFieldAssistiveSemantics {
                active_descendant: Some(NodeId::default()),
                active_descendant_element: Some(7),
                controls_element: Some(42),
                expanded: Some(true),
            },
            ..Default::default()
        });
    assert!(!draft_controller.is_bound());
}

#[test]
fn text_field_option_defaults_match_buffered_plain_text_baseline() {
    let options = TextFieldOptions::default();
    assert!(options.buffered);
    assert_eq!(options.blur_behavior, TextFieldBlurBehavior::Commit);
    assert_eq!(options.mode, TextFieldMode::PlainText);
    assert!(options.draft_controller.is_none());
    assert!(options.submit_command.is_none());
    assert_eq!(
        options.assistive_semantics,
        TextFieldAssistiveSemantics::default()
    );
}
