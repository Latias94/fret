use super::harness::{
    InputTextLifecycleScenario, LIFECYCLE_BLUR_TARGET_TEST_ID, LIFECYCLE_TEST_ID,
};
use super::*;

#[test]
fn input_text_lifecycle_tracks_focus_edit_and_blur_edges() {
    let mut scenario = InputTextLifecycleScenario::new(360.0, 220.0);
    let model = scenario.insert_text_model(String::new());
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let _root = scenario.render_lifecycle_frame(
        "imui-input-text-lifecycle",
        &model,
        &activated,
        &deactivated,
        &edited,
        &after_edit,
        &text,
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    let input = scenario.point_for_test_id(LIFECYCLE_TEST_ID);
    scenario.click_at(input);

    scenario.advance_frame();
    let _root = scenario.render_lifecycle_frame(
        "imui-input-text-lifecycle",
        &model,
        &activated,
        &deactivated,
        &edited,
        &after_edit,
        &text,
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    scenario.text_input("hello");

    scenario.advance_frame();
    let _root = scenario.render_lifecycle_frame(
        "imui-input-text-lifecycle",
        &model,
        &activated,
        &deactivated,
        &edited,
        &after_edit,
        &text,
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello");

    let blur_target = scenario.point_for_test_id(LIFECYCLE_BLUR_TARGET_TEST_ID);
    scenario.click_at(blur_target);

    scenario.advance_frame();
    let _root = scenario.render_lifecycle_frame(
        "imui-input-text-lifecycle",
        &model,
        &activated,
        &deactivated,
        &edited,
        &after_edit,
        &text,
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello");
    assert_eq!(scenario.model_text(&model).as_deref(), Some("hello"));
}
