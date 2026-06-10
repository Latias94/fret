use super::harness::InputTextCommandScenario;
use super::*;

#[test]
fn input_text_completion_command_dispatches_on_unmodified_tab() {
    let mut scenario = InputTextCommandScenario::new();
    let model = scenario.insert_text_model(String::new());
    let completion = fret_runtime::CommandId::from("editor.complete");

    let root = scenario.render_input(
        "imui-input-text-completion-command",
        &model,
        InputTextOptions {
            completion_command: Some(completion.clone()),
            test_id: Some(Arc::from("imui-input-text-completion-command")),
            ..Default::default()
        },
    );

    scenario.click_input(root);
    scenario.clear_effects();
    key_down(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    assert!(
        scenario
            .commands_for_window()
            .iter()
            .any(|command| command == &completion),
        "expected focused InputText Tab to dispatch the completion command"
    );
    assert_eq!(scenario.model_text(&model).as_deref(), Some(""));
}
