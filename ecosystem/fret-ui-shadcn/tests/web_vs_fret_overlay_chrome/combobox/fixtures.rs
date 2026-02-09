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
enum ComboboxOverlayChromeRecipe {
    HighlightedOptionChrome,
    FocusedOptionChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct ComboboxOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: ComboboxOverlayChromeRecipe,
    theme: WebThemeName,
}

#[test]
fn web_vs_fret_combobox_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_combobox_cases_v1.json"
    ));
    let suite: FixtureSuite<ComboboxOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome combobox fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome combobox case={}", case.id);
        match case.recipe {
            ComboboxOverlayChromeRecipe::HighlightedOptionChrome => {
                assert_listbox_highlighted_option_chrome_matches_web(
                    &case.web_name,
                    case.theme.as_str(),
                    "command-item",
                    case.theme.scheme(),
                    build_shadcn_combobox_demo_page,
                );
            }
            ComboboxOverlayChromeRecipe::FocusedOptionChrome => {
                assert_listbox_focused_option_chrome_matches_web(
                    &case.web_name,
                    case.theme.as_str(),
                    "command-item",
                    case.theme.scheme(),
                    build_shadcn_combobox_demo_page,
                    "Select a fruit",
                );
            }
        }
    }
}
