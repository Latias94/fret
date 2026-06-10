use super::harness::{InputTextModeScenario, PASSWORD_TEST_ID};
use super::*;

#[test]
fn input_text_password_mode_obscures_paint_text_without_mutating_model() {
    let mut scenario = InputTextModeScenario::new(320.0, 140.0);
    let model = scenario.insert_text_model("secret");

    let _root = scenario.render_input(
        PASSWORD_TEST_ID,
        &model,
        InputTextOptions {
            mode: InputTextMode::Password,
            test_id: Some(Arc::from(PASSWORD_TEST_ID)),
            ..Default::default()
        },
        None,
    );

    scenario.clear_prepared_text();
    scenario.paint_all();

    assert!(
        scenario
            .prepared_texts()
            .iter()
            .any(|text| text == "••••••"),
        "expected password mode to paint an obscured string"
    );
    assert_eq!(
        scenario.model_text(&model).as_deref(),
        Some("secret"),
        "expected password mode to preserve the underlying model value"
    );
}
