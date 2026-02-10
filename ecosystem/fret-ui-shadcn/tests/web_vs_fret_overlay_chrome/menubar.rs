use super::*;

#[path = "menubar/fixtures.rs"]
mod fixtures;

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
