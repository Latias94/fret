use super::*;

#[path = "context_menu/fixtures.rs"]
mod fixtures;

fn build_shadcn_context_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry,
        ContextMenuItem, ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItemSpec,
        ContextMenuShortcut,
    };

    ContextMenu::new(open.clone())
        // new-york-v4 context-menu-demo: `ContextMenuContent className="w-52"`.
        .min_width(Px(208.0))
        // new-york-v4 context-menu-demo: `ContextMenuSubContent className="w-44"`.
        .submenu_min_width(Px(176.0))
        .into_element(
            cx,
            |cx| {
                Button::new("Right click here")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                vec![
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Back")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘[").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Forward")
                            .inset(true)
                            .disabled(true)
                            .trailing(ContextMenuShortcut::new("⌘]").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Reload")
                            .inset(true)
                            .trailing(ContextMenuShortcut::new("⌘R").into_element(cx)),
                    ),
                    ContextMenuEntry::Item(ContextMenuItem::new("More Tools").inset(true).submenu(
                        vec![
                            ContextMenuEntry::Item(ContextMenuItem::new("Save Page...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Create Shortcut...")),
                            ContextMenuEntry::Item(ContextMenuItem::new("Name Window...")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Developer Tools")),
                            ContextMenuEntry::Separator,
                            ContextMenuEntry::Item(ContextMenuItem::new("Delete").variant(
                                fret_ui_shadcn::context_menu::ContextMenuItemVariant::Destructive,
                            )),
                        ],
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_bookmarks,
                        "Show Bookmarks",
                    )),
                    ContextMenuEntry::CheckboxItem(ContextMenuCheckboxItem::new(
                        checked_full_urls,
                        "Show Full URLs",
                    )),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Label(ContextMenuLabel::new("People").inset(true)),
                    ContextMenuEntry::RadioGroup(
                        ContextMenuRadioGroup::new(radio_person)
                            .item(ContextMenuRadioItemSpec::new("pedro", "Pedro Duarte"))
                            .item(ContextMenuRadioItemSpec::new("colm", "Colm Tuite")),
                    ),
                ]
            },
        )
}

fn build_shadcn_context_menu_demo_stateful(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    #[derive(Default)]
    struct Models {
        checked_bookmarks: Option<Model<bool>>,
        checked_full_urls: Option<Model<bool>>,
        radio_person: Option<Model<Option<Arc<str>>>>,
    }

    let existing = cx.with_state(Models::default, |st| {
        match (
            st.checked_bookmarks.as_ref(),
            st.checked_full_urls.as_ref(),
            st.radio_person.as_ref(),
        ) {
            (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
            _ => None,
        }
    });

    let (checked_bookmarks, checked_full_urls, radio_person) = if let Some(existing) = existing {
        existing
    } else {
        let checked_bookmarks = cx.app.models_mut().insert(false);
        let checked_full_urls = cx.app.models_mut().insert(true);
        let radio_person = cx.app.models_mut().insert(Some(Arc::from("benoit")));

        cx.with_state(Models::default, |st| {
            st.checked_bookmarks = Some(checked_bookmarks.clone());
            st.checked_full_urls = Some(checked_full_urls.clone());
            st.radio_person = Some(radio_person.clone());
        });

        (checked_bookmarks, checked_full_urls, radio_person)
    };

    build_shadcn_context_menu_demo(cx, open, checked_bookmarks, checked_full_urls, radio_person)
}

fn render_context_menu_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    open: &Model<bool>,
    checked_bookmarks: Model<bool>,
    checked_full_urls: Model<bool>,
    radio_person: Model<Option<Arc<str>>>,
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
                vec![build_shadcn_context_menu_demo(
                    cx,
                    open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_context_menu_highlighted_item_chrome_matches_web(
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

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let back = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node");

    hover_open_at(&mut ui, &mut app, &mut services, bounds_center(back.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let back = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, back.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted menu item background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text =
        find_best_text_color_near(&scene, back.bounds, leftish_text_probe_point(back.bounds))
            .unwrap_or_else(|| {
                panic!("{web_name} {web_theme_name}: highlighted menu item text color")
            });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_focused_item_chrome_matches_web(
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

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let back = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Back"))
        .expect("context-menu Back item semantics node");

    ui.set_focus(Some(back.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = fret_find_active_menu_item(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected focused menu item semantics node\n  focused_roles={focused_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
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
        focused.bounds,
        leftish_text_probe_point(focused.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused menu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused menu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_highlighted_item_chrome_matches_web(
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

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let (snap3, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        200,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );
    let save_page = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("context-menu submenu item (Save Page...) semantics node");

    ui.set_focus(Some(save_page.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(350),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let save_page = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Save Page..."))
        .expect("context-menu submenu item (Save Page...) semantics node after focus");

    let quad = find_best_solid_quad_within_matching_bg(&scene, save_page.bounds, expected.bg)
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
        save_page.bounds,
        leftish_text_probe_point(save_page.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: highlighted submenu item text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted submenu item text color"),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_destructive_focused_item_chrome_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("context-menu-demo.submenu-kbd-delete-focus");
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

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    // Settle the submenu open motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(500 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_context_menu_demo(
                    cx,
                    &open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }

    let snap3 = ui
        .semantics_snapshot()
        .expect("semantics snapshot after submenu open")
        .clone();
    let delete = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Delete"))
        .expect("context-menu submenu destructive Delete item semantics node");

    ui.set_focus(Some(delete.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(600),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let focused = fret_find_active_menu_item(&snap).unwrap_or_else(|| {
        panic!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: expected focused menu item semantics node"
        )
    });
    assert_eq!(
        focused.label.as_deref(),
        Some("Delete"),
        "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: focused menu item label"
    );

    let quad = find_best_quad_within_matching_bg(&scene, focused.bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!(
                "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: destructive focused menu item background quad"
            )
        });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name} destructive focused menu item background"
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
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name}: destructive focused menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd-delete-focus {web_theme_name} destructive focused menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}

fn assert_context_menu_submenu_destructive_item_idle_fg_matches_web(
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open("context-menu-demo.submenu-kbd");
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_menu_item_chrome_by_slot_variant_and_text(
        theme,
        "context-menu-item",
        "destructive",
        "Delete",
    );
    assert!(
        expected.bg.a <= 0.01,
        "context-menu-demo.submenu-kbd {web_theme_name}: expected destructive item bg to be transparent, got={expected:?}"
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

    let open: Model<bool> = app.models_mut().insert(false);
    let checked_bookmarks: Model<bool> = app.models_mut().insert(false);
    let checked_full_urls: Model<bool> = app.models_mut().insert(true);
    let radio_person: Model<Option<Arc<str>>> = app.models_mut().insert(Some(Arc::from("pedro")));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (snap1, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap1
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Right click here"))
        .expect("context-menu trigger semantics node");

    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let (snap2, _) = render_context_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
        checked_bookmarks.clone(),
        checked_full_urls.clone(),
        radio_person.clone(),
    );

    let more_tools = snap2
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("More Tools"))
        .expect("context-menu More Tools item semantics node");
    ui.set_focus(Some(more_tools.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    // Settle the submenu open motion.
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(500 + tick),
            tick + 1 == settle_frames,
            |cx| {
                vec![build_shadcn_context_menu_demo(
                    cx,
                    &open,
                    checked_bookmarks.clone(),
                    checked_full_urls.clone(),
                    radio_person.clone(),
                )]
            },
        );
    }

    let snap3 = ui
        .semantics_snapshot()
        .expect("semantics snapshot after submenu open")
        .clone();
    let delete = snap3
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some("Delete"))
        .expect("context-menu submenu destructive Delete item semantics node");
    assert!(
        !delete.flags.focused,
        "context-menu-demo.submenu-kbd {web_theme_name}: expected Delete to be idle (not focused)"
    );

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(600),
        true,
        |cx| {
            vec![build_shadcn_context_menu_demo(
                cx,
                &open,
                checked_bookmarks.clone(),
                checked_full_urls.clone(),
                radio_person.clone(),
            )]
        },
    );

    let (_, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let text = find_best_text_color_near(
        &scene,
        delete.bounds,
        leftish_text_probe_point(delete.bounds),
    )
    .unwrap_or_else(|| {
        panic!(
            "context-menu-demo.submenu-kbd {web_theme_name}: destructive idle menu item text color"
        )
    });
    assert_rgba_close(
        &format!(
            "context-menu-demo.submenu-kbd {web_theme_name} destructive idle menu item text color"
        ),
        text,
        expected.fg,
        0.03,
    );
}
