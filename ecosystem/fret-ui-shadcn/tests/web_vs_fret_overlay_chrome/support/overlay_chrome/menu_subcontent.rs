use super::*;

pub(crate) fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    hover_open_at(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(submenu_trigger.bounds),
    );

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

pub(crate) fn assert_menu_subcontent_surface_colors_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_background = web_portal
        .computed_style
        .get("backgroundColor")
        .and_then(|v| parse_css_color(v));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_border_color = web_portal
        .computed_style
        .get("borderTopColor")
        .and_then(|v| parse_css_color(v));
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    if let Some(web_background) = web_background
        && web_background.a > 0.01
    {
        let fret_bg = color_to_rgba(quad.background);
        assert_close(
            &format!("{web_name} {web_theme_name} background.r"),
            fret_bg.r,
            web_background.r,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.g"),
            fret_bg.g,
            web_background.g,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.b"),
            fret_bg.b,
            web_background.b,
            0.02,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} background.a"),
            fret_bg.a,
            web_background.a,
            0.02,
        );
    }

    if has_border(&web_border)
        && let Some(web_border_color) = web_border_color
        && web_border_color.a > 0.01
    {
        let fret_border = color_to_rgba(quad.border_color);
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.r"),
            fret_border.r,
            web_border_color.r,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.g"),
            fret_border.g,
            web_border_color.g,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.b"),
            fret_border.b,
            web_border_color.b,
            0.03,
        );
        assert_close(
            &format!("{web_name} {web_theme_name} border_color.a"),
            fret_border.a,
            web_border_color.a,
            0.03,
        );
    }
}

pub(crate) fn assert_menu_subcontent_shadow_insets_match_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let expected = web_drop_shadow_insets(web_portal);
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

pub(crate) fn assert_menu_subcontent_panel_size_matches_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));

    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (_snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let quad = find_best_chrome_quad_by_size(&scene, web_w, web_h, web_border)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    assert_close(
        &format!("{web_name} {web_theme_name} panel.w"),
        quad.rect.size.width.0,
        web_w,
        1.0,
    );
    assert_close(
        &format!("{web_name} {web_theme_name} panel.h"),
        quad.rect.size.height.0,
        web_h,
        1.0,
    );
}

pub(crate) fn assert_menu_subtrigger_open_chrome_matches_web_by_portal_slot_theme_keyboard_submenu(
    web_name: &str,
    web_subtrigger_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    bounds: Rect,
    root_settle_frames: u64,
    submenu_settle_frames: u64,
    open_action: impl FnOnce(
        &mut UiTree<App>,
        &mut App,
        &mut dyn fret_core::UiServices,
        Rect,
        &Model<bool>,
    ),
    submenu_trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected =
        web_find_open_menu_subtrigger_chrome(theme, web_subtrigger_slot, submenu_trigger_label);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let open: Model<bool> = app.models_mut().insert(false);

    let build_frame1 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build_frame1(cx, &open)],
    );

    open_action(&mut ui, &mut app, &mut services, bounds, &open);

    for tick in 0..root_settle_frames.max(1) {
        let request_semantics = tick + 1 == root_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let submenu_trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });
    ui.set_focus(Some(submenu_trigger.id));
    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowRight);

    let frame_base = 2 + root_settle_frames.max(1);
    for tick in 0..submenu_settle_frames.max(1) {
        let request_semantics = tick + 1 == submenu_settle_frames.max(1);
        let build_frame = build.clone();
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame_base + tick),
            request_semantics,
            |cx| vec![build_frame(cx, &open)],
        );
    }

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let trigger = snap
        .nodes
        .iter()
        .find(|n| {
            n.role == SemanticsRole::MenuItem && n.label.as_deref() == Some(submenu_trigger_label)
        })
        .unwrap_or_else(|| {
            panic!("missing submenu trigger semantics node: MenuItem {submenu_trigger_label:?}")
        });

    let quad = find_best_solid_quad_within_matching_bg(&scene, trigger.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: subtrigger open background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} subtrigger open background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        trigger.bounds,
        leftish_text_probe_point(trigger.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: subtrigger open text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} subtrigger open text color"),
        text,
        expected.fg,
        0.03,
    );
}
