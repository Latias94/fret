use super::harness::{
    InputTextModeScenario, SELECT_ALL_FIRST_TEST_ID, SELECT_ALL_SECOND_TEST_ID, SELECT_ALL_TEST_ID,
};
use super::*;

#[test]
fn input_text_select_all_on_focus_enables_copy() {
    let mut scenario = InputTextModeScenario::new(320.0, 140.0);
    let model = scenario.insert_text_model("select me");

    let root = scenario.render_input(
        SELECT_ALL_TEST_ID,
        &model,
        InputTextOptions {
            select_all_on_focus: true,
            test_id: Some(Arc::from(SELECT_ALL_TEST_ID)),
            ..Default::default()
        },
        None,
    );

    let at = scenario.first_child_point(root);
    scenario.click_at(at);

    scenario.advance_frame();
    let _root = scenario.render_input(
        SELECT_ALL_TEST_ID,
        &model,
        InputTextOptions {
            select_all_on_focus: true,
            test_id: Some(Arc::from(SELECT_ALL_TEST_ID)),
            ..Default::default()
        },
        None,
    );

    let dispatched = scenario.dispatch_all_timers();
    assert!(
        dispatched > 0,
        "expected select-all-on-focus timer to dispatch"
    );
    scenario.advance_frame();
    let _root = scenario.render_input(
        SELECT_ALL_TEST_ID,
        &model,
        InputTextOptions {
            select_all_on_focus: true,
            test_id: Some(Arc::from(SELECT_ALL_TEST_ID)),
            ..Default::default()
        },
        None,
    );
    let mut selected_all = false;
    for effect in scenario.drain_effects() {
        if let Effect::Command {
            window: Some(target_window),
            command,
        } = effect
            && target_window == scenario.window()
            && command == fret_runtime::CommandId::from("edit.select_all")
        {
            selected_all = scenario.dispatch_command(&command);
        }
    }
    assert!(
        selected_all,
        "expected focus-time timer to emit and dispatch edit.select_all"
    );
    assert!(
        scenario.is_command_available(&fret_runtime::CommandId::from("edit.copy")),
        "expected focus-time select_all to make copy available"
    );
}

#[test]
fn input_text_select_all_on_focus_drops_if_focus_moves_before_timer() {
    let mut scenario = InputTextModeScenario::new(360.0, 180.0);
    let first = scenario.insert_text_model("first");
    let second = scenario.insert_text_model("second");

    let _root =
        scenario.render_select_all_pair("imui-input-text-select-all-focus-move", &first, &second);

    let first_at = scenario.point_for_test_id(SELECT_ALL_FIRST_TEST_ID);
    scenario.click_at(first_at);

    scenario.advance_frame();
    let _root =
        scenario.render_select_all_pair("imui-input-text-select-all-focus-move", &first, &second);

    let second_at = scenario.point_for_test_id(SELECT_ALL_SECOND_TEST_ID);
    scenario.click_at(second_at);

    let dispatched = scenario.dispatch_all_timers();
    assert!(
        dispatched > 0,
        "expected select-all-on-focus timer to dispatch"
    );

    scenario.advance_frame();
    let _root =
        scenario.render_select_all_pair("imui-input-text-select-all-focus-move", &first, &second);

    assert!(
        !scenario.effects().iter().any(|effect| {
            matches!(
                effect,
                Effect::Command { command, .. }
                    if command == &fret_runtime::CommandId::from("edit.select_all")
            )
        }),
        "stale select-all-on-focus timer must not select text in the newly focused control"
    );
}
