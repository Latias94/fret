use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WebThemeName {
    Light,
    Dark,
}

impl WebThemeName {
    fn as_str(&self) -> &'static str {
        match self {
            WebThemeName::Light => "light",
            WebThemeName::Dark => "dark",
        }
    }

    fn scheme(&self) -> fret_ui_shadcn::shadcn_themes::ShadcnColorScheme {
        match self {
            WebThemeName::Light => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
            WebThemeName::Dark => fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ButtonGroupOverlayChromeRecipe {
    DestructiveItemIdleFg,
    DestructiveFocusedItemChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct ButtonGroupOverlayChromeCase {
    id: String,
    recipe: ButtonGroupOverlayChromeRecipe,
    theme: WebThemeName,
}

#[test]
fn web_vs_fret_button_group_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_button_group_cases_v1.json"
    ));
    let suite: FixtureSuite<ButtonGroupOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome button-group fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome button-group case={}", case.id);
        match case.recipe {
            ButtonGroupOverlayChromeRecipe::DestructiveItemIdleFg => {
                assert_button_group_demo_dropdown_menu_destructive_item_idle_fg_matches_web(
                    case.theme.as_str(),
                    case.theme.scheme(),
                );
            }
            ButtonGroupOverlayChromeRecipe::DestructiveFocusedItemChrome => {
                assert_button_group_demo_dropdown_menu_destructive_focused_item_chrome_matches_web(
                    case.theme.as_str(),
                    case.theme.scheme(),
                );
            }
        }
    }
}
