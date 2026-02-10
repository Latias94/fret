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

fn build_shadcn_menubar_demo(cx: &mut ElementContext<'_, App>) -> AnyElement {
    use fret_ui_shadcn::{
        Menubar, MenubarCheckboxItem, MenubarEntry, MenubarItem, MenubarMenu, MenubarRadioGroup,
        MenubarRadioItemSpec, MenubarShortcut,
    };

    #[derive(Default)]
    struct Models {
        view_bookmarks_bar: Option<Model<bool>>,
        view_full_urls: Option<Model<bool>>,
        profile_value: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| {
        match (
            st.view_bookmarks_bar.as_ref(),
            st.view_full_urls.as_ref(),
            st.profile_value.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
            _ => None,
        }
    });

    let (view_bookmarks_bar, view_full_urls, profile_value) = if let Some(existing) = existing {
        existing
    } else {
        let view_bookmarks_bar = cx.app.models_mut().insert(false);
        let view_full_urls = cx.app.models_mut().insert(true);
        let profile_value = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.view_bookmarks_bar = Some(view_bookmarks_bar.clone());
            st.view_full_urls = Some(view_full_urls.clone());
            st.profile_value = Some(profile_value.clone());
        });

        (view_bookmarks_bar, view_full_urls, profile_value)
    };

    Menubar::new(vec![
        MenubarMenu::new("File").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("New Tab")
                    .test_id("menubar.file.new_tab")
                    .trailing(MenubarShortcut::new("⌘T").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("New Window")
                    .trailing(MenubarShortcut::new("⌘N").into_element(cx)),
            ),
            MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(
                MenubarItem::new("Share")
                    .test_id("menubar.file.share")
                    .submenu(vec![
                        MenubarEntry::Item(MenubarItem::new("Email link")),
                        MenubarEntry::Item(MenubarItem::new("Messages")),
                        MenubarEntry::Item(MenubarItem::new("Notes")),
                    ]),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
            ),
        ]),
        MenubarMenu::new("Edit").entries(vec![
            MenubarEntry::Item(
                MenubarItem::new("Undo").trailing(MenubarShortcut::new("⌘Z").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Redo").trailing(MenubarShortcut::new("⇧⌘Z").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Submenu(MenubarItem::new("Find").submenu(vec![
                MenubarEntry::Item(MenubarItem::new("Search the web")),
                MenubarEntry::Separator,
                MenubarEntry::Item(MenubarItem::new("Find...")),
                MenubarEntry::Item(MenubarItem::new("Find Next")),
                MenubarEntry::Item(MenubarItem::new("Find Previous")),
            ])),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Cut")),
            MenubarEntry::Item(MenubarItem::new("Copy")),
            MenubarEntry::Item(MenubarItem::new("Paste")),
        ]),
        MenubarMenu::new("View").entries(vec![
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_bookmarks_bar,
                "Always Show Bookmarks Bar",
            )),
            MenubarEntry::CheckboxItem(MenubarCheckboxItem::new(
                view_full_urls,
                "Always Show Full URLs",
            )),
            MenubarEntry::Separator,
            MenubarEntry::Item(
                MenubarItem::new("Reload")
                    .inset(true)
                    .trailing(MenubarShortcut::new("⌘R").into_element(cx)),
            ),
            MenubarEntry::Item(
                MenubarItem::new("Force Reload")
                    .disabled(true)
                    .inset(true)
                    .trailing(MenubarShortcut::new("⇧⌘R").into_element(cx)),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Toggle Fullscreen").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Hide Sidebar").inset(true)),
        ]),
        MenubarMenu::new("Profiles").entries(vec![
            MenubarEntry::RadioGroup(
                MenubarRadioGroup::new(profile_value)
                    .item(MenubarRadioItemSpec::new("andy", "Andy"))
                    .item(MenubarRadioItemSpec::new("benoit", "Benoit"))
                    .item(MenubarRadioItemSpec::new("Luis", "Luis")),
            ),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Edit...").inset(true)),
            MenubarEntry::Separator,
            MenubarEntry::Item(MenubarItem::new("Add Profile...").inset(true)),
        ]),
    ])
    .into_element(cx)
}

fn render_shadcn_menubar_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| vec![build_shadcn_menubar_demo(cx)],
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    ui.set_focus(Some(file_trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 201);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .unwrap_or_else(|| {
            let focused_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.flags.focused)
                .filter_map(|n| n.label.as_deref())
                .collect();
            let menu_item_labels: Vec<&str> = snap
                .nodes
                .iter()
                .filter(|n| n.role == SemanticsRole::MenuItem)
                .filter_map(|n| n.label.as_deref())
                .collect();
            panic!(
                "{web_name} {web_theme_name}: expected menubar menu item 'New Tab'\n  focused_labels={focused_labels:?}\n  menu_item_labels={menu_item_labels:?}",
            )
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: focused menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn build_shadcn_menubar_file_menu_destructive(
    cx: &mut ElementContext<'_, App>,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> AnyElement {
    use fret_ui_shadcn::{Menubar, MenubarEntry, MenubarItem, MenubarMenu, MenubarShortcut};

    let mut new_tab =
        MenubarItem::new("New Tab").trailing(MenubarShortcut::new("⌘T").into_element(cx));
    if new_tab_destructive {
        new_tab = new_tab.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    let mut new_window =
        MenubarItem::new("New Window").trailing(MenubarShortcut::new("⌘N").into_element(cx));
    if new_window_destructive {
        new_window = new_window.variant(fret_ui_shadcn::menubar::MenubarItemVariant::Destructive);
    }

    Menubar::new(vec![MenubarMenu::new("File").entries(vec![
        MenubarEntry::Item(new_tab),
        MenubarEntry::Item(new_window),
        MenubarEntry::Item(MenubarItem::new("New Incognito Window").disabled(true)),
        MenubarEntry::Separator,
        MenubarEntry::Submenu(MenubarItem::new("Share").submenu(vec![
            MenubarEntry::Item(MenubarItem::new("Email link")),
            MenubarEntry::Item(MenubarItem::new("Messages")),
            MenubarEntry::Item(MenubarItem::new("Notes")),
        ])),
        MenubarEntry::Separator,
        MenubarEntry::Item(
            MenubarItem::new("Print...").trailing(MenubarShortcut::new("⌘P").into_element(cx)),
        ),
    ])])
    .into_element(cx)
}

fn render_shadcn_menubar_file_menu_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    new_tab_destructive: bool,
    new_window_destructive: bool,
) -> (fret_core::SemanticsSnapshot, Scene) {
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            ui,
            app,
            services,
            window,
            bounds,
            FrameId(frame_id_base + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_menubar_file_menu_destructive(
                    cx,
                    new_tab_destructive,
                    new_window_destructive,
                )]
            },
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_menubar_file_menu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-idle");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "menubar-item",
        "destructive",
        "New Window ⌘N",
    );
    assert!(
        expected.bg.a < 0.02,
        "menubar-demo.destructive-idle {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
    );

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, false, true)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        false,
        true,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        false,
        true,
    );

    let new_window = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Window"))
        .expect("menubar New Window item semantics node");
    assert!(
        !new_window.flags.focused,
        "menubar-demo.destructive-idle {web_theme_name}: expected New Window to be idle (not focused)"
    );

    let text = find_best_text_color_near(
        &scene,
        new_window.bounds,
        leftish_text_probe_point(new_window.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-idle {web_theme_name}: destructive idle menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-idle {web_theme_name} destructive idle menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_file_menu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("menubar-demo.destructive-focus-first");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_file_menu_destructive(cx, true, false)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        true,
        false,
    );

    let new_tab = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");
    ui.set_focus(Some(new_tab.id));

    let (snap, scene) = render_shadcn_menubar_file_menu_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        true,
        false,
    );

    let focused_fallback = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(focused_fallback);
    assert!(
        focused.role == SemanticsRole::MenuItem && focused.label.as_deref() == Some("New Tab"),
        "menubar-demo.destructive-focus-first {web_theme_name}: expected focused menu item to be New Tab, got role={:?} label={:?}",
        focused.role,
        focused.label
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item background"
        ),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "menubar-demo.destructive-focus-first {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "menubar-demo.destructive-focus-first {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(1440.0), Px(900.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node");

    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(new_tab.bounds),
    );

    let (snap, scene) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);

    let new_tab = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("New Tab"))
        .expect("menubar New Tab item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, new_tab.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        new_tab.bounds,
        leftish_text_probe_point(new_tab.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_menubar_submenu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = theme.viewport.map(bounds_for_viewport).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(375.0), Px(240.0)),
        )
    });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let file_trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("File"))
        .expect("menubar File trigger semantics node");

    left_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(file_trigger.bounds),
    );

    let (snap2, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 2);

    let share = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Share"))
        .expect("menubar submenu trigger (Share) semantics node");
    ui.set_focus(Some(share.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let (snap3, _) =
        render_shadcn_menubar_demo_settled(&mut ui, &mut app, &mut services, window, bounds, 200);
    let email_link = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node");
    ui.set_focus(Some(email_link.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(350),
        true,
        |cx| vec![build_shadcn_menubar_demo(cx)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let email_link = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Email link"))
        .expect("menubar submenu item (Email link) semantics node after focus");

    let quad = find_best_solid_quad_within_matching_bg(&scene, email_link.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted submenu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        email_link.bounds,
        leftish_text_probe_point(email_link.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted submenu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item text color"),
        text,
        expected.fg,
        0.03,
    );
}
