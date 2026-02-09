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
enum HoverCardOverlayChromeRecipe {
    PanelChrome,
    DemoPanelSize,
    SurfaceColors,
}

#[derive(Debug, Clone, Deserialize)]
struct HoverCardOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: HoverCardOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_hover_card(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    let trigger_el = fret_ui_shadcn::Button::new("@nextjs")
        .variant(fret_ui_shadcn::ButtonVariant::Link)
        .into_element(cx);
    let content_el =
        fret_ui_shadcn::HoverCardContent::new(vec![cx.text("@nextjs")]).into_element(cx);

    fret_ui_shadcn::HoverCard::new(trigger_el, content_el)
        .open(Some(open.clone()))
        .into_element(cx)
}

#[test]
fn web_vs_fret_hover_card_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_hover_card_cases_v1.json"
    ));
    let suite: FixtureSuite<HoverCardOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome hover-card fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome hover-card case={}", case.id);
        match case.recipe {
            HoverCardOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches_by_portal_slot(
                    &case.web_name,
                    "hover-card-content",
                    build_hover_card,
                );
            }
            HoverCardOverlayChromeRecipe::DemoPanelSize => {
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_overlay_panel_size_matches_by_portal_slot_theme_size_only(
                    &case.web_name,
                    "hover-card-content",
                    theme.as_str(),
                    theme.scheme(),
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_hover_card_demo_page,
                );
            }
            HoverCardOverlayChromeRecipe::SurfaceColors => {
                let theme = case.theme.as_ref().expect("surface_colors requires theme");
                assert_overlay_chrome_matches_by_portal_slot_theme(
                    &case.web_name,
                    "hover-card-content",
                    theme.as_str(),
                    theme.scheme(),
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_hover_card,
                );
            }
        }
    }
}
