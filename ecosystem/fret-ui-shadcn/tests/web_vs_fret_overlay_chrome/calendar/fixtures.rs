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
enum CalendarOverlayChromeBuilder {
    Calendar22,
    Calendar23,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CalendarOverlayChromeRecipe {
    PanelSize,
}

#[derive(Debug, Clone, Deserialize)]
struct CalendarOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: CalendarOverlayChromeRecipe,
    theme: WebThemeName,
    builder: CalendarOverlayChromeBuilder,
}

#[test]
fn web_vs_fret_calendar_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_calendar_cases_v1.json"
    ));
    let suite: FixtureSuite<CalendarOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome calendar fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome calendar case={}", case.id);
        match case.recipe {
            CalendarOverlayChromeRecipe::PanelSize => {
                let build = match case.builder {
                    CalendarOverlayChromeBuilder::Calendar22 => build_shadcn_calendar_22_page,
                    CalendarOverlayChromeBuilder::Calendar23 => build_shadcn_calendar_23_page,
                };
                assert_overlay_panel_size_matches_by_portal_slot_theme_with_tol(
                    &case.web_name,
                    "popover-content",
                    case.theme.as_str(),
                    case.theme.scheme(),
                    SemanticsRole::Panel,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    2.0,
                    build,
                );
            }
        }
    }
}
