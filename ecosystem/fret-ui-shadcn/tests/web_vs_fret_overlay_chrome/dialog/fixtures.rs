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
enum DialogOverlayChromeRecipe {
    PanelChrome,
    SurfaceColorsDefault,
    SurfaceColorsDark,
}

#[derive(Debug, Clone, Deserialize)]
struct DialogOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: DialogOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_dialog_demo(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    Dialog::new(open.clone()).into_element(
        cx,
        |cx| {
            Button::new("Open Dialog")
                .variant(ButtonVariant::Outline)
                .into_element(cx)
        },
        |cx| {
            DialogContent::new(vec![cx.text("Edit profile")])
                .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                .into_element(cx)
        },
    )
}

#[test]
fn web_vs_fret_dialog_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_dialog_cases_v1.json"
    ));
    let suite: FixtureSuite<DialogOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome dialog fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome dialog case={}", case.id);
        match case.recipe {
            DialogOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "dialog",
                    SemanticsRole::Dialog,
                    build_dialog_demo,
                );
            }
            DialogOverlayChromeRecipe::SurfaceColorsDefault => {
                assert_overlay_chrome_matches_by_portal_slot(
                    &case.web_name,
                    "dialog-content",
                    build_dialog_demo,
                );
            }
            DialogOverlayChromeRecipe::SurfaceColorsDark => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("surface_colors_dark requires theme");
                assert_overlay_chrome_matches_by_portal_slot_theme(
                    &case.web_name,
                    "dialog-content",
                    theme.as_str(),
                    theme.scheme(),
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_dialog_demo,
                );
            }
        }
    }
}
