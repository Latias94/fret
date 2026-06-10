use super::harness::InputTextCommandScenario;
use super::*;

#[test]
fn input_text_history_commands_dispatch_on_unmodified_arrows_without_default_repeat() {
    let mut scenario = InputTextCommandScenario::new();
    let model = scenario.insert_text_model(String::new());
    let previous = fret_runtime::CommandId::from("editor.history.previous");
    let next = fret_runtime::CommandId::from("editor.history.next");

    let root = scenario.render_input(
        "imui-input-text-history-commands",
        &model,
        InputTextOptions {
            history_previous_command: Some(previous.clone()),
            history_next_command: Some(next.clone()),
            test_id: Some(Arc::from("imui-input-text-history-commands")),
            ..Default::default()
        },
    );

    scenario.click_input(root);
    scenario.clear_effects();
    key_down(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::ArrowUp,
        Modifiers::default(),
    );
    key_down(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::ArrowDown,
        Modifiers::default(),
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
    assert_eq!(commands, vec![previous, next]);
}
