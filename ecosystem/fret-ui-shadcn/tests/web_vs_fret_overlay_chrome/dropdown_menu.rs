use super::*;

#[path = "dropdown_menu/fixtures.rs"]
mod fixtures;

fn build_shadcn_dropdown_menu_demo(
    cx: &mut ElementContext<'_, App>,
    open: &Model<bool>,
) -> AnyElement {
    use fret_ui_shadcn::{
        Button, ButtonVariant, DropdownMenu, DropdownMenuEntry, DropdownMenuItem,
        DropdownMenuLabel, DropdownMenuShortcut,
    };

    DropdownMenu::new(open.clone())
        // new-york-v4 dropdown-menu-demo: `DropdownMenuContent className="w-56"`.
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
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Billing")
                            .trailing(DropdownMenuShortcut::new("⌘B").into_element(cx)),
                    ),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Settings")
                            .trailing(DropdownMenuShortcut::new("⌘S").into_element(cx)),
                    ),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Keyboard shortcuts")
                            .trailing(DropdownMenuShortcut::new("⌘K").into_element(cx)),
                    ),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Team")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Invite users").submenu(vec![
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Email")),
                        DropdownMenuEntry::Item(DropdownMenuItem::new("Message")),
                        DropdownMenuEntry::Separator,
                        DropdownMenuEntry::Item(DropdownMenuItem::new("More...")),
                    ])),
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("New Team")
                            .trailing(DropdownMenuShortcut::new("⌘+T").into_element(cx)),
                    ),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(DropdownMenuItem::new("GitHub")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("Support")),
                    DropdownMenuEntry::Item(DropdownMenuItem::new("API").disabled(true)),
                    DropdownMenuEntry::Separator,
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new("Log out")
                            .trailing(DropdownMenuShortcut::new("⇧⌘Q").into_element(cx)),
                    ),
                ]
            },
        )
}

fn render_dropdown_menu_demo_settled(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    frame_id_base: u64,
    open: &Model<bool>,
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
            |cx| vec![build_shadcn_dropdown_menu_demo(cx, open)],
        );
    }
    paint_frame(ui, app, services, bounds)
}

fn assert_dropdown_menu_highlighted_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_menu_item_chrome(theme);

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
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

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let (snap2, _) = render_dropdown_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
    );

    let menu = largest_semantics_node(&snap2, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap2, menu.bounds)
        .expect("dropdown-menu first menu item semantics node");

    hover_open_at(&mut ui, &mut app, &mut services, bounds_center(item.bounds));

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menu = largest_semantics_node(&snap, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap, menu.bounds)
        .expect("dropdown-menu first menu item semantics node after hover");

    let quad = find_best_solid_quad_within_matching_bg(&scene, item.bounds, expected.bg)
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
        find_best_text_color_near(&scene, item.bounds, leftish_text_probe_point(item.bounds))
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

fn assert_dropdown_menu_focused_item_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_active_element_chrome(theme);

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
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

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let (snap2, _) = render_dropdown_menu_demo_settled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        2,
        &open,
    );

    let menu = largest_semantics_node(&snap2, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let item = fret_find_topmost_menu_item_in_menu(&snap2, menu.bounds)
        .expect("dropdown-menu first menu item semantics node");

    ui.set_focus(Some(item.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(200),
        true,
        |cx| vec![build_shadcn_dropdown_menu_demo(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let menu = largest_semantics_node(&snap, SemanticsRole::Menu)
        .expect("dropdown-menu menu semantics node (largest)");
    let fallback = fret_find_topmost_menu_item_in_menu(&snap, menu.bounds)
        .expect("dropdown-menu first menu item semantics node after focus");
    let focused = fret_find_active_menu_item(&snap).unwrap_or(fallback);
    let focused = focused;
    if focused.role != SemanticsRole::MenuItem {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        panic!(
            "{web_name} {web_theme_name}: expected focused menu item semantics node\n  focused_roles={focused_roles:?}"
        );
    }

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
