use super::*;

#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_surface_colors_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_panel_size_matches_web() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo.vp1440x240",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_shadow_matches_web_dark() {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    assert_overlay_shadow_insets_match(
        "dropdown-menu-demo",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_vp375x240_panel_size_matches_web_light() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo.vp375x240",
        "dropdown-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_vp375x240_panel_size_matches_web_dark() {
    assert_overlay_panel_size_matches_by_portal_slot_theme(
        "dropdown-menu-demo.vp375x240",
        "dropdown-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_dropdown_menu_demo,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_chrome_matches_web() {
    assert_dropdown_menu_highlighted_item_chrome_matches_web(
        "dropdown-menu-demo.highlight-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_chrome_matches_web_dark() {
    assert_dropdown_menu_highlighted_item_chrome_matches_web(
        "dropdown-menu-demo.highlight-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web() {
    assert_dropdown_menu_focused_item_chrome_matches_web(
        "dropdown-menu-demo.focus-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web_dark() {
    assert_dropdown_menu_focused_item_chrome_matches_web(
        "dropdown-menu-demo.focus-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_dropdown_menu_highlighted_item_chrome_matches_web(
        "dropdown-menu-demo.highlight-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_highlighted_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_dropdown_menu_highlighted_item_chrome_matches_web(
        "dropdown-menu-demo.highlight-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_dropdown_menu_focused_item_chrome_matches_web(
        "dropdown-menu-demo.focus-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_focused_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_dropdown_menu_focused_item_chrome_matches_web(
        "dropdown-menu-demo.focus-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "dropdown-menu-demo.submenu",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
        "dropdown-menu-demo.submenu",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_panel_size_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_subtrigger_open_chrome_matches_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-trigger",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_subtrigger_open_chrome_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-trigger",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(900.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, DropdownMenu, DropdownMenuEntry, DropdownMenuItem};

    let web = read_web_golden_open("dropdown-menu-demo.submenu-kbd-vp1440x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(1440.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "dropdown-menu-demo.submenu-kbd-vp1440x240",
        "dropdown-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |_ui, app, _services, _bounds, open| {
            let _ = app.models_mut().update(open, |v| *v = true);
        },
        "Invite users",
        |cx, open| {
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
        },
    );
}
#[test]
fn web_vs_fret_dropdown_menu_panel_chrome_matches() {
    assert_overlay_chrome_matches(
        "dropdown-menu-demo",
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
