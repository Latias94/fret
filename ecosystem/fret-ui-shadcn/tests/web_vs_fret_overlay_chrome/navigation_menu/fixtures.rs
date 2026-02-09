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
enum NavigationMenuOverlayChromeRecipe {
    PanelChrome,
    ContentSurfaceColors,
    ViewportSurfaceColors,
    ViewportShadowInsets,
    TriggerSurfaceColors,
    IndicatorShadowInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct NavigationMenuOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: NavigationMenuOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
    #[serde(default)]
    trigger_label: Option<String>,
    #[serde(default)]
    trigger_id: Option<String>,
    #[serde(default)]
    list_label: Option<String>,
}

fn build_navigation_menu_demo(
    cx: &mut ElementContext<'_, App>,
    model: &Model<Option<Arc<str>>>,
    root_id_out: &Rc<Cell<Option<GlobalElementId>>>,
    viewport: bool,
    indicator: bool,
) -> AnyElement {
    use fret_ui_shadcn::{NavigationMenu, NavigationMenuItem};

    let el = NavigationMenu::new(model.clone())
        .viewport(viewport)
        .indicator(indicator)
        .items(vec![NavigationMenuItem::new(
            "home",
            "Home",
            vec![cx.text("Content")],
        )])
        .into_element(cx);
    root_id_out.set(Some(el.id));
    el
}

#[test]
fn web_vs_fret_navigation_menu_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_navigation_menu_cases_v1.json"
    ));
    let suite: FixtureSuite<NavigationMenuOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome navigation-menu fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome navigation-menu case={}", case.id);
        match case.recipe {
            NavigationMenuOverlayChromeRecipe::PanelChrome => {
                assert_navigation_menu_content_chrome_matches(
                    &case.web_name,
                    "navigation-menu-content",
                    "open",
                    "home",
                    "Home",
                    |cx, model, root_id_out| {
                        build_navigation_menu_demo(cx, model, root_id_out, false, false)
                    },
                );
            }
            NavigationMenuOverlayChromeRecipe::ContentSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("content_surface_colors requires theme");
                assert_navigation_menu_content_surface_colors_match(
                    &case.web_name,
                    "navigation-menu-content",
                    "open",
                    "home",
                    "Home",
                    theme.as_str(),
                    theme.scheme(),
                    |cx, model, root_id_out| {
                        build_navigation_menu_demo(cx, model, root_id_out, false, false)
                    },
                );
            }
            NavigationMenuOverlayChromeRecipe::ViewportSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("viewport_surface_colors requires theme");
                assert_navigation_menu_viewport_surface_colors_match(
                    &case.web_name,
                    "navigation-menu-viewport",
                    "open",
                    "Home",
                    theme.as_str(),
                    theme.scheme(),
                    |cx, model, root_id_out| {
                        build_navigation_menu_demo(cx, model, root_id_out, true, true)
                    },
                );
            }
            NavigationMenuOverlayChromeRecipe::ViewportShadowInsets => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("viewport_shadow_insets requires theme");
                assert_navigation_menu_viewport_shadow_insets_match(
                    &case.web_name,
                    "navigation-menu-viewport",
                    "open",
                    "Home",
                    theme.as_str(),
                    theme.scheme(),
                    |cx, model, root_id_out| {
                        build_navigation_menu_demo(cx, model, root_id_out, true, true)
                    },
                );
            }
            NavigationMenuOverlayChromeRecipe::TriggerSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("trigger_surface_colors requires theme");
                let trigger_label = case
                    .trigger_label
                    .as_deref()
                    .expect("trigger_surface_colors requires trigger_label");
                let trigger_id = case
                    .trigger_id
                    .as_deref()
                    .expect("trigger_surface_colors requires trigger_id");
                let list_label = case
                    .list_label
                    .as_deref()
                    .expect("trigger_surface_colors requires list_label");

                assert_navigation_menu_trigger_surface_colors_match(
                    &case.web_name,
                    trigger_label,
                    trigger_id,
                    list_label,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            NavigationMenuOverlayChromeRecipe::IndicatorShadowInsets => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("indicator_shadow_insets requires theme");
                assert_navigation_menu_indicator_shadow_insets_match(
                    &case.web_name,
                    "navigation-menu-indicator",
                    "visible",
                    "Home",
                    theme.as_str(),
                    theme.scheme(),
                    |cx, model, root_id_out| {
                        build_navigation_menu_demo(cx, model, root_id_out, true, true)
                    },
                );
            }
        }
    }
}
