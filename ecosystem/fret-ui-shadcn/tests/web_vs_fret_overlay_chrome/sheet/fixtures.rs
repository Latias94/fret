use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize)]
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
enum SheetOverlayChromeRecipe {
    DemoPanelChrome,
    DemoSurfaceColorsDefault,
    DemoSurfaceColorsDark,
    SidePanelChrome,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SheetSideName {
    Top,
    Right,
    Bottom,
    Left,
}

impl SheetSideName {
    fn side(&self) -> fret_ui_shadcn::SheetSide {
        match self {
            SheetSideName::Top => fret_ui_shadcn::SheetSide::Top,
            SheetSideName::Right => fret_ui_shadcn::SheetSide::Right,
            SheetSideName::Bottom => fret_ui_shadcn::SheetSide::Bottom,
            SheetSideName::Left => fret_ui_shadcn::SheetSide::Left,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct SheetOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: SheetOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
    #[serde(default)]
    side: Option<SheetSideName>,
    #[serde(default)]
    trigger: Option<String>,
}

fn build_sheet_demo(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    Sheet::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
    )
}

fn build_sheet_side(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    side: fret_ui_shadcn::SheetSide,
    label: &str,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    Sheet::new(open.clone()).side(side).into_element(
        cx,
        |cx| {
            Button::new(label)
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
    )
}

#[test]
fn web_vs_fret_sheet_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_sheet_cases_v1.json"
    ));
    let suite: FixtureSuite<SheetOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome sheet fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome sheet case={}", case.id);
        match case.recipe {
            SheetOverlayChromeRecipe::DemoPanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_sheet_demo,
                );
            }
            SheetOverlayChromeRecipe::DemoSurfaceColorsDefault => {
                assert_overlay_chrome_matches_by_portal_slot(
                    &case.web_name,
                    "sheet-content",
                    build_sheet_demo,
                );
            }
            SheetOverlayChromeRecipe::DemoSurfaceColorsDark => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_surface_colors_dark requires theme");
                assert_overlay_surface_colors_match(
                    &case.web_name,
                    "sheet-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Dialog,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
                    build_sheet_demo,
                );
            }
            SheetOverlayChromeRecipe::SidePanelChrome => {
                let side = case.side.expect("side_panel_chrome requires side");
                let trigger = case
                    .trigger
                    .as_deref()
                    .expect("side_panel_chrome requires trigger");
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    move |cx, open| build_sheet_side(cx, open, side.side(), trigger),
                );
            }
        }
    }
}
