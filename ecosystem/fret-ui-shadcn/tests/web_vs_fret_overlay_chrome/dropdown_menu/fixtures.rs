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
enum DropdownMenuOverlayChromeRecipe {
    DemoSurfaceColors,
    DemoPanelSize,
    DemoShadowInsets,
    HighlightedItemChrome,
    FocusedItemChrome,
    SubmenuSurfaceColors,
    SubmenuShadowInsets,
    SubmenuPanelSize,
    SubtriggerOpenChrome,
    SubmenuSurfaceColorsKbd,
    PanelChrome,
}

#[derive(Debug, Clone, Deserialize)]
struct DropdownMenuOverlayChromeCase {
    id: String,
    web_name: String,
    recipe: DropdownMenuOverlayChromeRecipe,
    #[serde(default)]
    theme: Option<WebThemeName>,
}

fn build_dropdown_menu_demo_minimal(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    DropdownMenu::new(open.clone())
        .min_width(Px(224.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    DropdownMenuEntry::Label(DropdownMenuLabel::new("My Account")),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Profile")
                            .trailing(DropdownMenuShortcut::new("⇧⌘P").into_element(cx)),
                    ),
                ]
            },
        )
}

fn build_dropdown_menu_invite_submenu(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    DropdownMenu::new(open.clone())
        .min_width(Px(224.0))
        .into_element(
            cx,
            |cx| Button::new("Open").into_element(cx),
            |_cx| {
                vec![DropdownMenuEntry::Item(
                    DropdownMenuItem::new("Invite users").submenu(vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                    ]),
                )]
            },
        )
}

fn bounds_for_web_theme(name: &str, theme: &WebThemeName) -> Rect {
    let web = read_web_golden_open(name);
    web.themes
        .get(theme.as_str())
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        })
}

#[test]
fn web_vs_fret_dropdown_menu_overlay_chrome_cases_match_web_fixtures() {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/overlay_chrome_dropdown_menu_cases_v1.json"
    ));
    let suite: FixtureSuite<DropdownMenuOverlayChromeCase> =
        serde_json::from_str(raw).expect("overlay chrome dropdown-menu fixture parse");
    assert_eq!(suite.schema_version, 1);
    assert!(!suite.cases.is_empty());

    for case in suite.cases {
        eprintln!("overlay-chrome dropdown-menu case={}", case.id);
        match case.recipe {
            DropdownMenuOverlayChromeRecipe::DemoSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_surface_colors requires theme");
                assert_overlay_surface_colors_match(
                    &case.web_name,
                    "dropdown-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    |cx, open| build_dropdown_menu_demo_minimal(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::DemoPanelSize => {
                let theme = case.theme.as_ref().expect("demo_panel_size requires theme");
                assert_overlay_panel_size_matches_by_portal_slot_theme(
                    &case.web_name,
                    "dropdown-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    build_shadcn_dropdown_menu_demo,
                );
            }
            DropdownMenuOverlayChromeRecipe::DemoShadowInsets => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("demo_shadow_insets requires theme");
                assert_overlay_shadow_insets_match(
                    &case.web_name,
                    "dropdown-menu-content",
                    theme.as_str(),
                    theme.scheme(),
                    SemanticsRole::Menu,
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
                    |cx, open| build_dropdown_menu_demo_minimal(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::HighlightedItemChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("highlighted_item_chrome requires theme");
                assert_dropdown_menu_highlighted_item_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            DropdownMenuOverlayChromeRecipe::FocusedItemChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("focused_item_chrome requires theme");
                assert_dropdown_menu_focused_item_chrome_matches_web(
                    &case.web_name,
                    theme.as_str(),
                    theme.scheme(),
                );
            }
            DropdownMenuOverlayChromeRecipe::SubmenuSurfaceColors => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_surface_colors requires theme");
                let bounds = bounds_for_web_theme(&case.web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
                    &case.web_name,
                    "dropdown-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    |_ui, app, _services, _bounds, open| {
                        let _ = app.models_mut().update(open, |v| *v = true);
                    },
                    "Invite users",
                    |cx, open| build_dropdown_menu_invite_submenu(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::SubmenuShadowInsets => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_shadow_insets requires theme");
                let bounds = bounds_for_web_theme(&case.web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
                    &case.web_name,
                    "dropdown-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    |_ui, app, _services, _bounds, open| {
                        let _ = app.models_mut().update(open, |v| *v = true);
                    },
                    "Invite users",
                    |cx, open| build_dropdown_menu_invite_submenu(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::SubmenuPanelSize => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_panel_size requires theme");
                let bounds = bounds_for_web_theme(&case.web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
                    &case.web_name,
                    "dropdown-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    |_ui, app, _services, _bounds, open| {
                        let _ = app.models_mut().update(open, |v| *v = true);
                    },
                    "Invite users",
                    |cx, open| build_dropdown_menu_invite_submenu(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::SubtriggerOpenChrome => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("subtrigger_open_chrome requires theme");
                let bounds = bounds_for_web_theme(&case.web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
                    &case.web_name,
                    "dropdown-menu-sub-trigger",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    |_ui, app, _services, _bounds, open| {
                        let _ = app.models_mut().update(open, |v| *v = true);
                    },
                    "Invite users",
                    |cx, open| build_dropdown_menu_invite_submenu(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::SubmenuSurfaceColorsKbd => {
                let theme = case
                    .theme
                    .as_ref()
                    .expect("submenu_surface_colors_kbd requires theme");
                let bounds = bounds_for_web_theme(&case.web_name, theme);
                let settle_frames =
                    fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
                assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
                    &case.web_name,
                    "dropdown-menu-sub-content",
                    theme.as_str(),
                    theme.scheme(),
                    bounds,
                    settle_frames,
                    settle_frames,
                    |_ui, app, _services, _bounds, open| {
                        let _ = app.models_mut().update(open, |v| *v = true);
                    },
                    "Invite users",
                    |cx, open| build_dropdown_menu_invite_submenu(cx, open),
                );
            }
            DropdownMenuOverlayChromeRecipe::PanelChrome => {
                assert_overlay_chrome_matches(
                    &case.web_name,
                    "menu",
                    SemanticsRole::Menu,
                    |cx, open| {
                        fret_ui_shadcn::DropdownMenu::new(open.clone()).into_element(
                            cx,
                            |cx| fret_ui_shadcn::Button::new("Open").into_element(cx),
                            |_cx| {
                                vec![fret_ui_shadcn::DropdownMenuEntry::Item(
                                    fret_ui_shadcn::DropdownMenuItem::new("Alpha"),
                                )]
                            },
                        )
                    },
                );
            }
        }
    }
}
