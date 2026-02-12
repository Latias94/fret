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
enum ContextMenuOverlayChromeRecipe {
    DemoSurfaceColors,
    HighlightedItemChrome,
    FocusedItemChrome,
    SubmenuHighlightedItemChrome,
    SubmenuDestructiveFocusedItemChrome,
    SubmenuDestructiveItemIdleFg,
    SubmenuSurfaceColors,
    SubmenuKeyboardShadowInsets,
    SubmenuKeyboardPanelSize,
    SubmenuKeyboardSurfaceColors,
    SubtriggerOpenChromeKeyboard,
    PanelChrome,
    DemoPanelSize,
    DemoShadowInsets,
}

#[derive(Debug, Clone, Deserialize)]
struct ContextMenuOverlayChromeCase {
    id: String,
    #[serde(default)]
    web_name: Option<String>,
    recipe: ContextMenuOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_context_menu_copy(cx: &mut ElementContext<'_, App>, open: &Model<bool>) -> AnyElement {
    fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
        cx,
        |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
        |_cx| {
            vec![fret_ui_shadcn::ContextMenuEntry::Item(
                fret_ui_shadcn::ContextMenuItem::new("Copy"),
            )]
        },
    )
}

fn build_context_menu_copy_with_demo_widths(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    ContextMenu::new(open.clone())
        .min_width(Px(208.0))
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
            |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
        )
}

fn build_context_menu_more_tools_submenu(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    ContextMenu::new(open.clone())
        .min_width(Px(208.0))
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| Button::new("Right click here").into_element(cx),
            |_cx| {
                vec![ContextMenuEntry::Item(
                    ContextMenuItem::new("More Tools").inset(true).submenu(vec![
                        ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                        ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                        ContextMenuEntry::Separator,
                        ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                            fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                        )),
                    ]),
                )]
            },
        )
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

fn right_click_open_context_menu(
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
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger");
    right_click_center(ui, app, services, bounds_center(trigger.bounds));
}

#[test]
fn web_vs_fret_context_menu_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_context_menu_cases_v1.json"
    ));
    let suite: FixtureSuite<ContextMenuOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome context menu fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome context-menu case={}", case.id);
        match case.recipe {
            ContextMenuOverlayChromeRecipe::DemoSurfaceColors => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("demo_surface_colors requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_surface_colors requires theme");
                assert_overlay_surface_colors_match(
                    web_name,
                    "context-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_context_menu_copy_with_demo_widths,
                );
            }
            ContextMenuOverlayChromeRecipe::HighlightedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("highlighted_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("highlighted_item_chrome requires theme");
                assert_context_menu_highlighted_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            ContextMenuOverlayChromeRecipe::FocusedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("focused_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("focused_item_chrome requires theme");
                assert_context_menu_focused_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuHighlightedItemChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("submenu_highlighted_item_chrome requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_highlighted_item_chrome requires theme");
                assert_context_menu_submenu_highlighted_item_chrome_matches_web(
                    web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuDestructiveFocusedItemChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_destructive_focused_item_chrome requires theme");
                assert_context_menu_submenu_destructive_focused_item_chrome_matches_web(
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuDestructiveItemIdleFg => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_destructive_item_idle_fg requires theme");
                assert_context_menu_submenu_destructive_item_idle_fg_matches_web(
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuSurfaceColors => {
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
                    "context-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    right_click_open_context_menu,
                    "More Tools",
                    build_context_menu_more_tools_submenu,
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuKeyboardShadowInsets => {
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
                    "context-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    right_click_open_context_menu,
                    "More Tools",
                    build_context_menu_more_tools_submenu,
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuKeyboardPanelSize => {
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
                    "context-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    right_click_open_context_menu,
                    "More Tools",
                    build_context_menu_more_tools_submenu,
                );
            }
            ContextMenuOverlayChromeRecipe::SubmenuKeyboardSurfaceColors => {
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
                    "context-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    right_click_open_context_menu,
                    "More Tools",
                    build_context_menu_more_tools_submenu,
                );
            }
            ContextMenuOverlayChromeRecipe::SubtriggerOpenChromeKeyboard => {
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
                    "context-menu-sub-trigger",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    right_click_open_context_menu,
                    "More Tools",
                    build_context_menu_more_tools_submenu,
                );
            }
            ContextMenuOverlayChromeRecipe::PanelChrome => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("panel_chrome requires web_name");
                assert_context_menu_chrome_matches(
                    web_name,
                    "menu",
                    SemanticsRole::Menu,
                    "Right click here",
                    build_context_menu_copy,
                );
            }
            ContextMenuOverlayChromeRecipe::DemoPanelSize => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("demo_panel_size requires web_name");
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_context_menu_panel_size_matches_by_portal_slot_theme(
                    web_name,
                    "context-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    "Right click here",
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_context_menu_demo_stateful,
                );
            }
            ContextMenuOverlayChromeRecipe::DemoShadowInsets => {
                let web_name = case
                    .web_name
                    .as_deref()
                    .expect("demo_shadow_insets requires web_name");
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_shadow_insets requires theme");
                assert_context_menu_shadow_insets_match(
                    web_name,
                    "context-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    "Right click here",
                    build_context_menu_copy,
                );
            }
        }
    }
}
