use super::harness::InputTextCommandScenario;
use super::*;

#[test]
fn input_text_policy_commands_can_opt_into_repeat() {
    let mut scenario = InputTextCommandScenario::new();
    let model = scenario.insert_text_model(String::new());
    let completion = fret_runtime::CommandId::from("editor.complete.repeat");
    let previous = fret_runtime::CommandId::from("editor.history.previous.repeat");
    let undo = fret_runtime::CommandId::from("editor.undo.repeat");

    let root = scenario.render_input(
        "imui-input-text-command-repeat",
        &model,
        InputTextOptions {
            completion_command: Some(completion.clone()),
            history_previous_command: Some(previous.clone()),
            undo_command: Some(undo.clone()),
            completion_command_repeat: true,
            history_command_repeat: true,
            undo_redo_command_repeat: true,
            test_id: Some(Arc::from("imui-input-text-command-repeat")),
            ..Default::default()
        },
    );

    scenario.click_input(root);
    scenario.clear_effects();
    key_down_with_repeat(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::Tab,
        Modifiers::default(),
        true,
    );
    key_down_with_repeat(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::KeyZ,
        ctrl_modifiers(),
        true,
    );
    key_down_with_repeat(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::ArrowUp,
        Modifiers::default(),
        true,
    );

    let commands = scenario.commands_for_window();
    assert_eq!(commands, vec![completion, undo, previous]);
}
