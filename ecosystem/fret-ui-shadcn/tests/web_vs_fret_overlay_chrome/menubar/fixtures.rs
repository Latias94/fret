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
enum MenubarClickBuilder {
    MinimalFileMenu,
    ShadcnDemo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MenubarOverlayChromeRecipe {
    PanelChrome,
    DemoPanelSize,
    ClickOverlayShadowInsets,
    ClickOverlaySurfaceColors,
    SubmenuSurfaceColors,
    SubmenuKeyboardShadowInsets,
    SubmenuKeyboardPanelSize,
    SubmenuKeyboardSurfaceColors,
    SubtriggerOpenChromeKeyboard,
    FocusedItemChrome,
    HighlightedItemChrome,
    FileMenuDestructiveItemIdleFg,
    FileMenuDestructiveFocusedItemChrome,
    SubmenuHighlightedItemChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct MenubarOverlayChromeCase {
    id: String,
    #[serde(default)]
    web_name: Option<String>,
    recipe: MenubarOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
    #[serde(default)]
    trigger: Option<String>,
    #[serde(default)]
    click_builder: Option<MenubarClickBuilder>,
}

fn build_minimal_file_menu(cx: &mut ElementContext<'_, App>) -> AnyElement {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    Menubar::new(vec![MenubarMenu::new("File").entries(vec![
        MenubarEntry::Item(MenubarItem::new("New Tab")),
        MenubarEntry::Item(MenubarItem::new("New Window")),
        MenubarEntry::Separator,
        MenubarEntry::Item(MenubarItem::new("Share")),
    ])])
    .into_element(cx)
}

fn build_share_submenu(cx: &mut ElementContext<'_, App>, _open: &Model<bool>) -> AnyElement {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    Menubar::new(vec![MenubarMenu::new("File").entries(vec![
        MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
            MenubarEntry::Item(MenubarItem::new("Email link")),
            MenubarEntry::Item(MenubarItem::new("Messages")),
            MenubarEntry::Item(MenubarItem::new("Notes")),
        ])),
    ])])
    .into_element(cx)
}

fn default_bounds_for_web_name(web_name: &str) -> Rect {
    let (w, h) = if web_name.contains("vp375x240") {
        (375.0, 240.0)
    } else if web_name.contains("vp1440x240") {
        (1440.0, 240.0)
    } else {
        (1440.0, 900.0)
    };

    Rect::new(Point::new(Px(0.0), Px(0.0)), CoreSize::new(Px(w), Px(h)))
}

fn bounds_for_web_theme_or(web_name: &str, theme: &WebThemeName) -> Rect {
    let web = read_web_golden_open(web_name);
    web.themes
        .get(theme.as_str())
        .and_then(bounds_for_theme_viewport)
        .unwrap_or_else(|| default_bounds_for_web_name(web_name))
}

fn click_open_file_menu(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    _bounds: Rect,
    _open: &Model<bool>,
) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger");
    left_click_center(ui, app, services, bounds_center(trigger.bounds));
}

#[test]
fn web_vs_fret_menubar_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_menubar_cases_v1.json"
    ));
    let suite: FixtureSuite<MenubarOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome menubar fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome menubar case={}", case.id);
        match case.recipe {
            MenubarOverlayChromeRecipe::PanelChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("panel_chrome requires web_name");
                assert_click_overlay_chrome_matches(
                    web_name,
                    "menu",
                    SemanticsRole::Menu,
                    SemanticsRole::MenuItem,
                    "File",
                    |cx| build_minimal_file_menu(cx),
                );
            }
            MenubarOverlayChromeRecipe::DemoPanelSize => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("demo_panel_size requires web_name");
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_click_overlay_panel_size_matches_by_portal_slot_theme(
                    web_name,
                    "menubar-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::MenuItem,
                    "File",
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_menubar_demo,
                );
            }
            MenubarOverlayChromeRecipe::ClickOverlayShadowInsets => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("click_overlay_shadow_insets requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("click_overlay_shadow_insets requires theme");
                let trigger = case
                    .trigger
                    .as_deref()
                    .expect("click_overlay_shadow_insets requires trigger");
                let builder = case
                    .click_builder
                    .as_ref()
                    .expect("click_overlay_shadow_insets requires click_builder");

                match builder {
                    MenubarClickBuilder::MinimalFileMenu => {
                        assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
                            web_name,
                            "menubar-content",
                            theme.as_str(),
                            theme.scheme(),
                            SemanticsRole::MenuItem,
                            trigger,
                            fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                            |cx| build_minimal_file_menu(cx),
                        );
                    }
                    MenubarClickBuilder::ShadcnDemo => {
                        assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
                            web_name,
                            "menubar-content",
                            theme.as_str(),
                            theme.scheme(),
                            SemanticsRole::MenuItem,
                            trigger,
                            fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                            build_shadcn_menubar_demo,
                        );
                    }
                }
            }
            MenubarOverlayChromeRecipe::ClickOverlaySurfaceColors => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("click_overlay_surface_colors requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("click_overlay_surface_colors requires theme");
                let trigger = case
                    .trigger
                    .as_deref()
                    .expect("click_overlay_surface_colors requires trigger");
                let builder = case
                    .click_builder
                    .as_ref()
                    .expect("click_overlay_surface_colors requires click_builder");

                match builder {
                    MenubarClickBuilder::MinimalFileMenu => {
                        assert_click_overlay_surface_colors_match_by_portal_slot_theme(
                            web_name,
                            "menubar-content",
                            theme.as_str(),
                            theme.scheme(),
                            SemanticsRole::MenuItem,
                            trigger,
                            fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                            |cx| build_minimal_file_menu(cx),
                        );
                    }
                    MenubarClickBuilder::ShadcnDemo => {
                        assert_click_overlay_surface_colors_match_by_portal_slot_theme(
                            web_name,
                            "menubar-content",
                            theme.as_str(),
                            theme.scheme(),
                            SemanticsRole::MenuItem,
                            trigger,
                            fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                            build_shadcn_menubar_demo,
                        );
                    }
                }
            }
            MenubarOverlayChromeRecipe::SubmenuSurfaceColors => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_surface_colors requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_surface_colors requires theme");
                let bounds = bounds_for_web_theme_or(web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
                    web_name,
                    "menubar-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    click_open_file_menu,
                    "Share",
                    build_share_submenu,
                );
            }
            MenubarOverlayChromeRecipe::SubmenuKeyboardShadowInsets => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_keyboard_shadow_insets requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_keyboard_shadow_insets requires theme");
                let bounds = bounds_for_web_theme_or(web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
                    web_name,
                    "menubar-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    click_open_file_menu,
                    "Share",
                    build_share_submenu,
                );
            }
            MenubarOverlayChromeRecipe::SubmenuKeyboardPanelSize => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_keyboard_panel_size requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_keyboard_panel_size requires theme");
                let bounds = bounds_for_web_theme_or(web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
                    web_name,
                    "menubar-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    click_open_file_menu,
                    "Share",
                    build_share_submenu,
                );
            }
            MenubarOverlayChromeRecipe::SubmenuKeyboardSurfaceColors => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_keyboard_surface_colors requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_keyboard_surface_colors requires theme");
                let bounds = bounds_for_web_theme_or(web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
                    web_name,
                    "menubar-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    click_open_file_menu,
                    "Share",
                    build_share_submenu,
                );
            }
            MenubarOverlayChromeRecipe::SubtriggerOpenChromeKeyboard => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("subtrigger_open_chrome_keyboard requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("subtrigger_open_chrome_keyboard requires theme");
                let bounds = bounds_for_web_theme_or(web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
                    web_name,
                    "menubar-sub-trigger",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    click_open_file_menu,
                    "Share",
                    build_share_submenu,
                );
            }
            MenubarOverlayChromeRecipe::FocusedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("focused_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("focused_item_chrome requires theme");
                assert_menubar_focused_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            MenubarOverlayChromeRecipe::HighlightedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("highlighted_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("highlighted_item_chrome requires theme");
                assert_menubar_highlighted_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            MenubarOverlayChromeRecipe::FileMenuDestructiveItemIdleFg => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("file_menu_destructive_item_idle_fg requires theme");
                assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            MenubarOverlayChromeRecipe::FileMenuDestructiveFocusedItemChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("file_menu_destructive_focused_item_chrome requires theme");
                assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            MenubarOverlayChromeRecipe::SubmenuHighlightedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_highlighted_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_highlighted_item_chrome requires theme");
                assert_menubar_submenu_highlighted_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
        }
    }
}
