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
enum SelectOverlayChromeRecipe {
    PanelChrome,
    SurfaceColors,
    ShadowInsets,
    DemoHighlightedOptionChrome,
    ScrollableHighlightedOptionChrome,
    DemoFocusedOptionChrome,
    ScrollableFocusedOptionChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct SelectOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: SelectOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_select_simple_scrollable(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    let value: Model<Option<Arc<str>>> = cx.app.models_mut().insert(None);
    fret_ui_shadcn::Select::new(value, open.clone())
        .a11y_label("Select")
        .item(fret_ui_shadcn::SelectItem::new("one", "One"))
        .item(fret_ui_shadcn::SelectItem::new("two", "Two"))
        .into_element(cx)
}

#[test]
fn web_vs_fret_select_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_select_cases_v1.json"
    ));
    let suite: FixtureSuite<SelectOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome select fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome select case={}", case.id);
        match case.recipe {
            SelectOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "listbox",
                    SemanticsRole::ListBox,
                    |cx, open| build_select_simple_scrollable(cx, open),
                );
            }
            SelectOverlayChromeRecipe::SurfaceColors => {
                let theme = case.theme.as_ref().expect("surface_colors requires theme");
                assert_overlay_surface_colors_match(
                    &case.web_name,
                    "select-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::ListBox,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_select_scrollable_demo,
                );
            }
            SelectOverlayChromeRecipe::ShadowInsets => {
                let theme = case.theme.as_ref().expect("shadow_insets requires theme");
                assert_overlay_shadow_insets_match(
                    &case.web_name,
                    "select-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::ListBox,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_select_scrollable_demo,
                );
            }
            SelectOverlayChromeRecipe::DemoHighlightedOptionChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_highlighted_option_chrome requires theme");
                assert_listbox_highlighted_option_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    "select-item",
                    theme.scheme(),
                    build_shadcn_select_demo_page,
                );
            }
            SelectOverlayChromeRecipe::ScrollableHighlightedOptionChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("scrollable_highlighted_option_chrome requires theme");
                assert_listbox_highlighted_option_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    "select-item",
                    theme.scheme(),
                    build_shadcn_select_scrollable_page,
                );
            }
            SelectOverlayChromeRecipe::DemoFocusedOptionChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_focused_option_chrome requires theme");
                assert_listbox_focused_option_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    "select-item",
                    theme.scheme(),
                    build_shadcn_select_demo_page,
                    "Select",
                );
            }
            SelectOverlayChromeRecipe::ScrollableFocusedOptionChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("scrollable_focused_option_chrome requires theme");
                assert_listbox_focused_option_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    "select-item",
                    theme.scheme(),
                    build_shadcn_select_scrollable_page,
                    "Select",
                );
            }
        }
    }
}
