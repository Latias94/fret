use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SelectFixtureRecipe {
    ScrollableOverlayPlacement,
    DemoOverlayPlacement,
    DemoOpenOptionMetrics,
    ScrollableListboxOptionInsets,
    ScrollableListboxOptionHeight,
    ScrollableListboxHeight,
    ScrollableScrollButtonHeight,
    ScrollableViewportInsets,
    ScrollableListboxWidth,
}

#[derive(Debug, Clone, Deserialize)]
struct SelectFixtureCase {
    id: String,
    web_name: String,
    recipe: SelectFixtureRecipe,
}

#[test]
fn web_vs_fret_select_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_select_cases_v1.json"
    ));
    let suite: FixtureSuite<SelectFixtureCase> =
        serde_json::from_str(raw).expect("select fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("select case={}", case.id);
        match case.recipe {
            SelectFixtureRecipe::ScrollableOverlayPlacement => {
                assert_select_scrollable_overlay_placement_matches_impl(&case.web_name);
            }
            SelectFixtureRecipe::DemoOverlayPlacement => {
                assert_select_demo_overlay_placement_matches_impl(&case.web_name);
            }
            SelectFixtureRecipe::DemoOpenOptionMetrics => {
                assert_select_demo_open_option_metrics_match(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableListboxOptionInsets => {
                assert_select_scrollable_listbox_option_insets_match(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableListboxOptionHeight => {
                assert_select_scrollable_listbox_option_height_matches(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableListboxHeight => {
                assert_select_scrollable_listbox_height_matches(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableScrollButtonHeight => {
                assert_select_scrollable_scroll_button_height_matches(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableViewportInsets => {
                assert_select_scrollable_viewport_insets_match(&case.web_name);
            }
            SelectFixtureRecipe::ScrollableListboxWidth => {
                assert_select_scrollable_listbox_width_matches(&case.web_name);
            }
        }
    }
}
