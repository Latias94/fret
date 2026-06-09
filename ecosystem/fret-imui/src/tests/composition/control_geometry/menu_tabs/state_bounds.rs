use super::harness::{MenuTabGeometryScenario, TOP_LEVEL_TRIGGER_IDS};

#[test]
fn menu_and_tab_trigger_state_changes_keep_outer_bounds_stable() {
    let mut scenario = MenuTabGeometryScenario::new();
    scenario.render_frame();

    let baseline = scenario.baseline_for(&TOP_LEVEL_TRIGGER_IDS);
    for test_id in TOP_LEVEL_TRIGGER_IDS {
        scenario.assert_top_level_trigger_state_bounds_stable(&baseline, test_id);
    }

    scenario.select_inspector_tab_and_assert_bounds(&baseline);
    scenario.open_file_menu_and_assert_bounds(&baseline);

    let submenu_test_id = "imui-geometry.menu.file.recent";
    let submenu_before = scenario.bounds_for(submenu_test_id);
    scenario.assert_submenu_state_bounds_stable(submenu_test_id, submenu_before);
}
