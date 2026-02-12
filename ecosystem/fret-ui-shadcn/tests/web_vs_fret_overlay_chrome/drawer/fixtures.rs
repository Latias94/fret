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
enum DrawerOverlayChromeRecipe {
    SurfaceColors,
}

#[derive(Debug, Clone, Deserialize)]
struct DrawerOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: DrawerOverlayChromeRecipe,
    theme: WebThemeName,
}

fn build_drawer_demo(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    Drawer::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open Drawer")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
    )
}

#[test]
fn web_vs_fret_drawer_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_drawer_cases_v1.json"
    ));
    let suite: FixtureSuite<DrawerOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome drawer fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome drawer case={}", case.id);
        match case.recipe {
            DrawerOverlayChromeRecipe::SurfaceColors => {
                assert_overlay_surface_colors_match(
                    &case.web_name,
                    "drawer-content",
                    case.theme.as_str(),
                    case.theme.scheme(),
                    SemanticsRole::Dialog,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
                    build_drawer_demo,
                );
            }
        }
    }
}
