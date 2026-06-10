use super::harness::InputTextCommandScenario;
use super::*;

#[test]
fn input_text_undo_redo_commands_dispatch_on_focused_shortcuts_without_default_repeat() {
    let mut scenario = InputTextCommandScenario::new();
    let model = scenario.insert_text_model(String::new());
    let undo = fret_runtime::CommandId::from("editor.undo");
    let redo = fret_runtime::CommandId::from("editor.redo");

    let root = scenario.render_input(
        "imui-input-text-undo-redo-commands",
        &model,
        InputTextOptions {
            undo_command: Some(undo.clone()),
            redo_command: Some(redo.clone()),
            test_id: Some(Arc::from("imui-input-text-undo-redo-commands")),
            ..Default::default()
        },
    );

    scenario.click_input(root);
    scenario.clear_effects();
    key_down_ctrl(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::KeyZ,
    );
    key_down_ctrl(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::KeyY,
    );
    key_down(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::KeyZ,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    key_down_with_repeat(
        &mut scenario.ui,
        &mut scenario.app,
        &mut scenario.services,
        KeyCode::KeyZ,
        ctrl_modifiers(),
        true,
    );

    let commands = scenario.commands_for_window();
    assert_eq!(commands, vec![undo, redo.clone(), redo]);
}
