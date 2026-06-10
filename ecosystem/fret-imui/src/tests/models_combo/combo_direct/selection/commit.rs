use super::harness::ComboDirectSelectionScenario;

#[test]
fn combo_can_commit_selection_with_selectable_rows() {
    let mut scenario = ComboDirectSelectionScenario::new();

    scenario.render_frame();
    assert!(scenario.selected().is_none());

    scenario.click_trigger();
    scenario.advance_frame();
    scenario.render_frame();
    assert!(scenario.has_first_option());

    scenario.click_first_option();
    scenario.advance_frame();
    scenario.render_frame();
    assert_eq!(scenario.selected().as_deref(), Some("Alpha"));

    scenario.advance_frame();
    scenario.render_frame();
    assert_eq!(scenario.selected().as_deref(), Some("Alpha"));
    assert!(!scenario.has_first_option());
}
