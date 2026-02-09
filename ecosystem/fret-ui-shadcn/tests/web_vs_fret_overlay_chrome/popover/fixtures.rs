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
enum PopoverOverlayChromeRecipe {
    DemoSurfaceColors,
    PanelChrome,
    DemoPanelSize,
}

#[derive(Debug, Clone, Deserialize)]
struct PopoverOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: PopoverOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_popover_surface_colors_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Popover, PopoverContent};

    Popover::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open popover")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            PopoverContent::new(Vec::new())
                .refine_layout(
                    fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(320.0))
                        .h_px(Px(245.33334)),
                )
                .into_element(cx)
        },
    )
}

fn build_popover_simple(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    fret_ui_shadcn::Popover::new(open.clone()).into_element(
        cx,
        |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
        |cx| fret_ui_shadcn::PopoverContent::new(Vec::new()).into_element(cx),
    )
}

#[test]
fn web_vs_fret_popover_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_popover_cases_v1.json"
    ));
    let suite: FixtureSuite<PopoverOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome popover fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome popover case={}", case.id);
        match case.recipe {
            PopoverOverlayChromeRecipe::DemoSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_surface_colors requires theme");
                assert_overlay_surface_colors_match(
                    &case.web_name,
                    "popover-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Dialog,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_popover_surface_colors_demo,
                );
            }
            PopoverOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_popover_simple,
                );
            }
            PopoverOverlayChromeRecipe::DemoPanelSize => {
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_overlay_panel_size_matches_by_portal_slot_theme(
                    &case.web_name,
                    "popover-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Dialog,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_popover_demo_page,
                );
            }
        }
    }
}
