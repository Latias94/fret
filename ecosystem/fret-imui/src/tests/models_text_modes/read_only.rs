use super::harness::{InputTextModeScenario, READ_ONLY_TEST_ID};
use super::*;

#[test]
fn input_text_read_only_blocks_text_input_and_keeps_changed_false() {
    let mut scenario = InputTextModeScenario::new(320.0, 140.0);
    let model = scenario.insert_text_model("locked");
    let changed = Rc::new(Cell::new(false));

    let root = scenario.render_input(
        READ_ONLY_TEST_ID,
        &model,
        InputTextOptions {
            read_only: true,
            test_id: Some(Arc::from(READ_ONLY_TEST_ID)),
            ..Default::default()
        },
        Some(changed.clone()),
    );
    assert!(!changed.get());

    let at = scenario.first_child_point(root);
    scenario.click_at(at);
    scenario.text_input("!");

    scenario.advance_frame();
    let _root = scenario.render_input(
        READ_ONLY_TEST_ID,
        &model,
        InputTextOptions {
            read_only: true,
            test_id: Some(Arc::from(READ_ONLY_TEST_ID)),
            ..Default::default()
        },
        Some(changed.clone()),
    );

    assert!(!changed.get());
    assert_eq!(scenario.model_text(&model).as_deref(), Some("locked"));
}
