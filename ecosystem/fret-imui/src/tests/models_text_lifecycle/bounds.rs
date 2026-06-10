use super::harness::{InputTextLifecycleScenario, STABLE_BOUNDS_TEST_ID};
use super::*;

#[test]
fn input_text_focus_keeps_control_bounds_stable() {
    let mut scenario = InputTextLifecycleScenario::new(320.0, 140.0);
    let model = scenario.insert_text_model(String::new());

    let _root = scenario.render_stable_bounds_frame("imui-input-text-stable-bounds", &model);
    let before = scenario.bounds_for_test_id(STABLE_BOUNDS_TEST_ID);

    let at = Point::new(
        Px(before.origin.x.0 + before.size.width.0 * 0.5),
        Px(before.origin.y.0 + before.size.height.0 * 0.5),
    );
    scenario.click_at(at);

    scenario.advance_frame();
    let _root = scenario.render_stable_bounds_frame("imui-input-text-stable-bounds", &model);
    let after = scenario.bounds_for_test_id(STABLE_BOUNDS_TEST_ID);

    assert_eq!(after.origin, before.origin);
    assert_eq!(after.size, before.size);
}
