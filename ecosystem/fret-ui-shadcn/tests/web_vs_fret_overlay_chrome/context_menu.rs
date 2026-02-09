use super::*;

#[test]
fn web_vs_fret_context_menu_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{ContextMenu, ContextMenuEntry, ContextMenuItem};

    assert_overlay_surface_colors_match(
        "context-menu-demo",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            ContextMenu::new(open.clone())
                .min_width(Px(208.0))
                .submenu_min_width(Px(176.0))
                .into_element(
                    cx,
                    |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                    |_cx| vec![ContextMenuEntry::Item(ContextMenuItem::new("Copy"))],
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_chrome_matches_web() {
    assert_context_menu_highlighted_item_chrome_matches_web(
        "context-menu-demo.highlight-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_chrome_matches_web_dark() {
    assert_context_menu_highlighted_item_chrome_matches_web(
        "context-menu-demo.highlight-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_focused_item_chrome_matches_web() {
    assert_context_menu_focused_item_chrome_matches_web(
        "context-menu-demo.focus-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_focused_item_chrome_matches_web_dark() {
    assert_context_menu_focused_item_chrome_matches_web(
        "context-menu-demo.focus-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_context_menu_highlighted_item_chrome_matches_web(
        "context-menu-demo.highlight-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_highlighted_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_context_menu_highlighted_item_chrome_matches_web(
        "context-menu-demo.highlight-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_focused_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_context_menu_focused_item_chrome_matches_web(
        "context-menu-demo.focus-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_focused_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_context_menu_focused_item_chrome_matches_web(
        "context-menu-demo.focus-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_highlight_first_vp375x240_item_chrome_matches_web() {
    assert_context_menu_submenu_highlighted_item_chrome_matches_web(
        "context-menu-demo.submenu-highlight-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_highlight_first_vp375x240_item_chrome_matches_web_dark() {
    assert_context_menu_submenu_highlighted_item_chrome_matches_web(
        "context-menu-demo.submenu-highlight-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_destructive_focused_item_chrome_matches_web() {
    assert_context_menu_submenu_destructive_focused_item_chrome_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_destructive_focused_item_chrome_matches_web_dark() {
    assert_context_menu_submenu_destructive_focused_item_chrome_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_destructive_item_idle_fg_matches_web() {
    assert_context_menu_submenu_destructive_item_idle_fg_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_destructive_item_idle_fg_matches_web_dark() {
    assert_context_menu_submenu_destructive_item_idle_fg_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu");
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
        "context-menu-demo.submenu",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu");
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
        "context-menu-demo.submenu",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_shadow_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_panel_size_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_panel_size_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("light")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_vp375x240_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp375x240");
    let bounds = web
        .themes
        .get("dark")
        .and_then(|t| t.viewport)
        .map(bounds_for_viewport)
        .unwrap_or_else(|| {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                CoreSize::new(Px(375.0), Px(240.0)),
            )
        });

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
        "context-menu-demo.submenu-kbd-vp375x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
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
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
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
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_subtrigger_open_chrome_matches_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-trigger",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_subtrigger_open_chrome_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-trigger",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
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
        "context-menu-demo.submenu-kbd",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
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
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ContextMenu, ContextMenuEntry, ContextMenuItem};

    let web = read_web_golden_open("context-menu-demo.submenu-kbd-vp1440x240");
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
        "context-menu-demo.submenu-kbd-vp1440x240",
        "context-menu-sub-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        bounds,
        settle_frames,
        settle_frames,
        |ui, app, services, _bounds, _open| {
            let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
            let trigger = snap
                .nodes
                .iter()
                .find(|n| {
                    n.role == SemanticsRole::Button
                        && n.label.as_deref() == Some("Right click here")
                })
                .expect("context-menu trigger");
            right_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "More Tools",
        |cx, open| {
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
                                ContextMenuEntry::Item(
                                    ContextMenuItem::new("Delete").variant(
                                        fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                                    ),
                                ),
                            ]),
                        )]
                    },
                )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_panel_chrome_matches() {
    assert_context_menu_chrome_matches(
        "context-menu-demo",
        "menu",
        SemanticsRole::Menu,
        "Right click here",
        |cx, open| {
            fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::ContextMenuEntry::Item(
                        fret_ui_shadcn::ContextMenuItem::new("Copy"),
                    )]
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_panel_size_matches_web() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_panel_size_matches_web_dark() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_panel_size_matches_web() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo.vp1440x240",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_vp375x240_panel_size_matches_web_light() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo.vp375x240",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_vp375x240_panel_size_matches_web_dark() {
    assert_context_menu_panel_size_matches_by_portal_slot_theme(
        "context-menu-demo.vp375x240",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        "Right click here",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_context_menu_demo_stateful,
    );
}
#[test]
fn web_vs_fret_context_menu_demo_shadow_matches_web() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo",
        "context-menu-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Menu,
        "Right click here",
        |cx, open| {
            fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::ContextMenuEntry::Item(
                        fret_ui_shadcn::ContextMenuItem::new("Copy"),
                    )]
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_context_menu_demo_shadow_matches_web_dark() {
    assert_context_menu_shadow_insets_match(
        "context-menu-demo",
        "context-menu-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Menu,
        "Right click here",
        |cx, open| {
            fret_ui_shadcn::ContextMenu::new(open.clone()).into_element(
                cx,
                |cx| fret_ui_shadcn::Button::new("Right click here").into_element(cx),
                |_cx| {
                    vec![fret_ui_shadcn::ContextMenuEntry::Item(
                        fret_ui_shadcn::ContextMenuItem::new("Copy"),
                    )]
                },
            )
        },
    );
}
