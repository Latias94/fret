use super::*;

pub(crate) fn assert_context_menu_chrome_matches(
    web_name: &str,
    web_portal_role: &str,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let web_portal = find_portal_by_role(theme, web_portal_role).expect("web portal root by role");
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_radius = web_corner_radii_effective_px(web_portal).expect("web radius px");

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

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

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);

    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

    let quad =
        find_best_chrome_quad(&scene, overlay.bounds).expect("painted quad for overlay panel");
    for (idx, edge) in quad.border.iter().enumerate() {
        assert_close(
            &format!("{web_name} border[{idx}]"),
            *edge,
            web_border[idx],
            0.6,
        );
    }
    for (idx, corner) in quad.corners.iter().enumerate() {
        assert_close(
            &format!("{web_name} radius[{idx}]"),
            *corner,
            web_radius[idx],
            1.0,
        );
    }
}

pub(crate) fn assert_context_menu_shadow_insets_match(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    trigger_label: &str,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let expected = web_drop_shadow_insets(web_portal);

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme_scheme(&mut app, scheme);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(640.0), Px(480.0)),
    );

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

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    let build_frame2 = build.clone();
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build_frame2(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));
    let quad = find_best_chrome_quad(&scene, overlay.bounds)
        .unwrap_or_else(|| panic!("painted quad for overlay panel ({web_name})"));

    let candidates = fret_drop_shadow_insets_candidates(&scene, quad.rect);
    assert_shadow_insets_match(web_name, web_theme_name, &expected, &candidates);
}

pub(crate) fn assert_context_menu_panel_size_matches_by_portal_slot_theme(
    web_name: &str,
    web_portal_slot: &str,
    web_theme_name: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    fret_role: SemanticsRole,
    trigger_label: &str,
    settle_frames: u64,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);

    let web_portal = find_portal_by_slot(theme, web_portal_slot)
        .unwrap_or_else(|| panic!("missing web portal slot={web_portal_slot} for {web_name}"));
    let web_border = web_border_widths_px(web_portal).expect("web border widths px");
    let web_w = web_portal.rect.w;
    let web_h = web_portal.rect.h;

    let bounds = bounds_for_theme_viewport(theme).unwrap_or_else(|| {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(640.0), Px(480.0)),
        )
    });

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

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(trigger_label))
        .unwrap_or_else(|| panic!("missing trigger semantics node: Button {trigger_label:?}"));
    right_click_center(
        &mut ui,
        &mut app,
        &mut services,
        bounds_center(trigger.bounds),
    );

    for tick in 0..settle_frames.max(1) {
        let request_semantics = tick + 1 == settle_frames.max(1);
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

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let _overlay = snap
        .nodes
        .iter()
        .find(|n| n.role == fret_role)
        .unwrap_or_else(|| panic!("missing fret semantics node: {fret_role:?}"));

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
