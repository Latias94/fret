use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModeToggleRecipe {
    ConstrainedMenuItemHeight,
    ConstrainedMenuContentInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct ModeToggleCase {
    id: String,
    web_name: String,
    recipe: ModeToggleRecipe,
}

#[test]
fn web_vs_fret_mode_toggle_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_mode_toggle_cases_v1.json"
    ));
    let suite: FixtureSuite<ModeToggleCase> =
        serde_json::from_str(raw).expect("mode-toggle fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("mode-toggle case={}", case.id);
        match case.recipe {
            ModeToggleRecipe::ConstrainedMenuItemHeight => {
                assert_mode_toggle_constrained_menu_item_height_matches(&case.web_name);
            }
            ModeToggleRecipe::ConstrainedMenuContentInsets => {
                assert_mode_toggle_constrained_menu_content_insets_match(&case.web_name);
            }
        }
    }
}
