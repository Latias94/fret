use super::*;
use serde::Deserialize;
use std::cell::Cell;
use std::rc::Rc;

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
enum TooltipOverlayChromeRecipe {
    PanelChrome,
    DemoPanelSize,
    DemoPanelHeight,
    SurfaceColors,
}

#[derive(Debug, Clone, Deserialize)]
struct TooltipOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: TooltipOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_tooltip_demo(
    cx: &mut ElementContext<'_, App>,
    trigger: &Rc<Cell<Option<GlobalElementId>>>,
) -> AnyElement {
    let trigger_el = fret_ui_shadcn::Button::new("Hover")
        .variant(fret_ui_shadcn::ButtonVariant::Outline)
        .into_element(cx);
    trigger.set(Some(trigger_el.id));

    let content_el =
        fret_ui_shadcn::TooltipContent::new(vec![fret_ui_shadcn::TooltipContent::text(
            cx,
            "Add to library",
        )])
        .into_element(cx);

    fret_ui_shadcn::Tooltip::new(trigger_el, content_el)
        .open_delay_frames(0)
        .close_delay_frames(0)
        .into_element(cx)
}

#[test]
fn web_vs_fret_tooltip_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_tooltip_cases_v1.json"
    ));
    let suite: FixtureSuite<TooltipOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome tooltip fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome tooltip case={}", case.id);
        match case.recipe {
            TooltipOverlayChromeRecipe::PanelChrome => {
                assert_hover_overlay_chrome_matches(
                    &case.web_name,
                    "tooltip-content",
                    SemanticsRole::Tooltip,
                    "Hover",
                    build_tooltip_demo,
                );
            }
            TooltipOverlayChromeRecipe::DemoPanelSize => {
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_hover_overlay_panel_size_matches_by_portal_slot_theme(
                    &case.web_name,
                    "tooltip-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Tooltip,
                    "Hover",
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_tooltip_demo,
                );
            }
            TooltipOverlayChromeRecipe::DemoPanelHeight => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_panel_height requires theme");
                assert_hover_overlay_panel_height_matches_by_portal_slot_theme(
                    &case.web_name,
                    "tooltip-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Tooltip,
                    "Hover",
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_tooltip_demo,
                );
            }
            TooltipOverlayChromeRecipe::SurfaceColors => {
                let theme = case.theme.as_ref().expect("surface_colors requires theme");
                assert_hover_overlay_surface_colors_match_by_portal_slot_theme(
                    &case.web_name,
                    "tooltip-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Tooltip,
                    "Hover",
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_tooltip_demo,
                );
            }
        }
    }
}
