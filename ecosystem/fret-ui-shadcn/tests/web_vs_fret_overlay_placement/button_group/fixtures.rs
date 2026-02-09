use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ButtonGroupRecipe {
    ConstrainedMenuItemHeight,
    ConstrainedMenuContentInsets,
    SubmenuOverlayPlacement,
    SubmenuMenuContentInsets,
    SubmenuMenuItemHeight,
}

#[derive(Debug, Clone, Deserialize)]
struct ButtonGroupCase {
    id: String,
    web_name: String,
    recipe: ButtonGroupRecipe,
}

#[test]
fn web_vs_fret_button_group_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_placement_button_group_cases_v1.json"
    ));
    let suite: FixtureSuite<ButtonGroupCase> =
        serde_json::from_str(raw).expect("button-group fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("button-group case={}", case.id);
        match case.recipe {
            ButtonGroupRecipe::ConstrainedMenuItemHeight => {
                assert_button_group_demo_constrained_menu_item_height_matches(&case.web_name);
            }
            ButtonGroupRecipe::ConstrainedMenuContentInsets => {
                assert_button_group_demo_constrained_menu_content_insets_match(&case.web_name);
            }
            ButtonGroupRecipe::SubmenuOverlayPlacement => {
                assert_button_group_demo_submenu_overlay_placement_matches(&case.web_name);
            }
            ButtonGroupRecipe::SubmenuMenuContentInsets => {
                assert_button_group_demo_submenu_constrained_menu_content_insets_match(
                    &case.web_name,
                );
            }
            ButtonGroupRecipe::SubmenuMenuItemHeight => {
                assert_button_group_demo_submenu_menu_item_height_matches(&case.web_name);
            }
        }
    }
}
