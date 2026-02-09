use super::*;

#[test]
fn web_vs_fret_menubar_panel_chrome_matches() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_chrome_matches(
        "menubar-demo",
        "menu",
        SemanticsRole::Menu,
        SemanticsRole::MenuItem,
        "File",
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_panel_size_matches_web() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_panel_size_matches_web_dark() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_panel_size_matches_web() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_tiny_viewport_panel_size_matches_web_dark() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo.vp1440x240",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_vp375x240_panel_size_matches_web_light() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo.vp375x240",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_vp375x240_panel_size_matches_web_dark() {
    assert_click_overlay_panel_size_matches_by_portal_slot_theme(
        "menubar-demo.vp375x240",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_view_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_view_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_profiles_shadow_matches_web() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_profiles_shadow_matches_web_dark() {
    assert_click_overlay_shadow_insets_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_root_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme_named(&web, "light");
    let web_root =
        find_by_data_slot(&theme.root, "menubar").expect("web menubar root node (data-slot)");
    let expected = web_drop_shadow_insets(web_root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                    MenubarEntry::Item(MenubarItem::new("New Tab")),
                    MenubarEntry::Item(MenubarItem::new("New Window")),
                    MenubarEntry::Separator,
                    MenubarEntry::Item(MenubarItem::new("Share")),
                ])])
                .into_element(cx),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menubar = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuBar)
        .expect("fret menubar semantics node");
    let quad = find_best_chrome_quad(&scene, menubar.bounds).expect("menubar chrome quad");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match("menubar-demo", "light", &expected, &candidates);
}
#[test]
fn web_vs_fret_menubar_root_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo");
    let theme = web_theme_named(&web, "dark");
    let web_root =
        find_by_data_slot(&theme.root, "menubar").expect("web menubar root node (data-slot)");
    let expected = web_drop_shadow_insets(web_root);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(
        &mut app,
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![
                Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                    MenubarEntry::Item(MenubarItem::new("New Tab")),
                    MenubarEntry::Item(MenubarItem::new("New Window")),
                    MenubarEntry::Separator,
                    MenubarEntry::Item(MenubarItem::new("Share")),
                ])])
                .into_element(cx),
            ]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menubar = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuBar)
        .expect("fret menubar semantics node");
    let quad = find_best_chrome_quad(&scene, menubar.bounds).expect("menubar chrome quad");

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match("menubar-demo", "dark", &expected, &candidates);
}
#[test]
fn web_vs_fret_menubar_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "File",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Item(MenubarItem::new("New Tab")),
                MenubarEntry::Item(MenubarItem::new("New Window")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Share")),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_view_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_view_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.view",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "View",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_profiles_surface_colors_match_web() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_profiles_surface_colors_match_web_dark() {
    assert_click_overlay_surface_colors_match_by_portal_slot_theme(
        "menubar-demo.profiles",
        "menubar-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::MenuItem,
        "Profiles",
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        build_shadcn_menubar_demo,
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu");
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
        "menubar-demo.submenu",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu");
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
        "menubar-demo.submenu",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_shadow_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_panel_size_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_panel_size_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_vp375x240_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp375x240");
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
        "menubar-demo.submenu-kbd-vp375x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_panel_size_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
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
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_panel_size_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
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
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_subtrigger_open_chrome_matches_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-trigger",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_subtrigger_open_chrome_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-trigger",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_shadow_matches_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd");
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
        "menubar-demo.submenu-kbd",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
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
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_kbd_tiny_viewport_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu};

    let web = read_web_golden_open("menubar-demo.submenu-kbd-vp1440x240");
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
        "menubar-demo.submenu-kbd-vp1440x240",
        "menubar-sub-content",
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
                .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
                .expect("menubar File trigger");
            left_click_center(ui, app, services, bounds_center(trigger.bounds));
        },
        "Share",
        |cx, _open| {
            Menubar::new(vec![MenubarMenu::new("File").entries(vec![
                MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
                    MenubarEntry::Item(MenubarItem::new("Email link")),
                    MenubarEntry::Item(MenubarItem::new("Messages")),
                    MenubarEntry::Item(MenubarItem::new("Notes")),
                ])),
            ])])
            .into_element(cx)
        },
    );
}
#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web() {
    assert_menubar_focused_item_chrome_matches_web(
        "menubar-demo.focus-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web_dark() {
    assert_menubar_focused_item_chrome_matches_web(
        "menubar-demo.focus-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_menubar_focused_item_chrome_matches_web(
        "menubar-demo.focus-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_focused_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_menubar_focused_item_chrome_matches_web(
        "menubar-demo.focus-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_destructive_item_idle_fg_matches_web() {
    assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_destructive_item_idle_fg_matches_web_dark() {
    assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_destructive_focused_item_chrome_matches_web() {
    assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_destructive_focused_item_chrome_matches_web_dark() {
    assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web() {
    assert_menubar_highlighted_item_chrome_matches_web(
        "menubar-demo.highlight-first",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web_dark() {
    assert_menubar_highlighted_item_chrome_matches_web(
        "menubar-demo.highlight-first",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web_mobile_tiny_viewport() {
    assert_menubar_highlighted_item_chrome_matches_web(
        "menubar-demo.highlight-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_highlighted_item_chrome_matches_web_dark_mobile_tiny_viewport() {
    assert_menubar_highlighted_item_chrome_matches_web(
        "menubar-demo.highlight-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_highlight_first_vp375x240_item_chrome_matches_web() {
    assert_menubar_submenu_highlighted_item_chrome_matches_web(
        "menubar-demo.submenu-highlight-first-vp375x240",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
    );
}
#[test]
fn web_vs_fret_menubar_demo_submenu_highlight_first_vp375x240_item_chrome_matches_web_dark() {
    assert_menubar_submenu_highlighted_item_chrome_matches_web(
        "menubar-demo.submenu-highlight-first-vp375x240",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );
}
