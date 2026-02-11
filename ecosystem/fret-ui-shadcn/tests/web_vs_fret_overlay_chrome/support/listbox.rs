use super::*;

fn hover_first_listbox_option(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
) {
    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let mut option_candidates: Vec<(Rect, &fret_core::SemanticsNode)> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .map(|n| (ui.debug_node_bounds(n.id).unwrap_or(n.bounds), n))
        .collect();
    option_candidates.sort_by(|(a, _), (b, _)| {
        a.origin
            .y
            .0
            .total_cmp(&b.origin.y.0)
            .then_with(|| a.origin.x.0.total_cmp(&b.origin.x.0))
    });

    let option = option_candidates
        .iter()
        .find(|(bounds, _)| rect_contains(listbox_bounds, *bounds))
        .map(|(_, n)| *n)
        .unwrap_or_else(|| {
            let samples: Vec<Rect> = option_candidates.iter().take(8).map(|(b, _)| *b).collect();
            panic!(
                "listbox option\n  listbox_bounds={listbox_bounds:?}\n  first_option_bounds={samples:?}"
            )
        });
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    ui.dispatch_event(
        app,
        services,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId(0),
            position: bounds_center(option_bounds),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );
}

pub(crate) fn assert_listbox_highlighted_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    web_option_slot: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, web_option_slot);

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

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );
    let _ = app.models_mut().update(&open, |v| *v = true);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(2 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    hover_first_listbox_option(&mut ui, &mut app, &mut services);
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2 + settle_frames),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let listbox = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBox)
        .max_by(|a, b| {
            rect_area(ui.debug_node_bounds(a.id).unwrap_or(a.bounds))
                .total_cmp(&rect_area(ui.debug_node_bounds(b.id).unwrap_or(b.bounds)))
        })
        .expect("listbox");
    let listbox_bounds = ui.debug_node_bounds(listbox.id).unwrap_or(listbox.bounds);
    let option = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| {
            rect_contains(
                listbox_bounds,
                ui.debug_node_bounds(n.id).unwrap_or(n.bounds),
            )
        })
        .min_by(|a, b| {
            let a_bounds = ui.debug_node_bounds(a.id).unwrap_or(a.bounds);
            let b_bounds = ui.debug_node_bounds(b.id).unwrap_or(b.bounds);
            a_bounds
                .origin
                .y
                .0
                .total_cmp(&b_bounds.origin.y.0)
                .then_with(|| a_bounds.origin.x.0.total_cmp(&b_bounds.origin.x.0))
        })
        .expect("listbox option");
    let option_bounds = ui.debug_node_bounds(option.id).unwrap_or(option.bounds);

    let quad = find_best_solid_quad_within_matching_bg(&scene, option_bounds, expected.bg)
        .unwrap_or_else(|| {
            panic!("{web_name} {web_theme_name}: highlighted option background quad")
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(&scene, listbox_bounds, bounds_center(option_bounds))
        .unwrap_or_else(|| {
            let mut total_text = 0usize;
            let mut samples_raw: Vec<(f32, f32)> = Vec::new();
            let mut samples_tx: Vec<(f32, f32)> = Vec::new();
            scene_walk(&scene, |st, op| {
                let SceneOp::Text { origin, .. } = *op else {
                    return;
                };
                total_text += 1;
                if samples_raw.len() < 16 {
                    samples_raw.push((origin.x.0, origin.y.0));
                }
                if samples_tx.len() < 16 {
                    let p = st.transform.apply_point(origin);
                    samples_tx.push((p.x.0, p.y.0));
                }
            });
            panic!(
                "{web_name} {web_theme_name}: highlighted option text color (no text ops near)\n  total_text_ops={total_text}\n  sample_origins_raw={samples_raw:?}\n  sample_origins_tx={samples_tx:?}\n  listbox_bounds={listbox_bounds:?}\n  option_bounds={option_bounds:?}",
            )
        });
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} highlighted option text color"),
        text,
        expected.fg,
        0.03,
    );
}

pub(crate) fn assert_listbox_focused_option_chrome_matches_web(
    web_name: &str,
    web_theme_name: &str,
    web_option_slot: &str,
    scheme: fret_ui_shadcn::shadcn_themes::ShadcnColorScheme,
    build: impl Fn(&mut ElementContext<'_, App>, &Model<bool>) -> AnyElement + Clone,
    a11y_label: &str,
) {
    let web = read_web_golden_open(web_name);
    let theme = web_theme_named(&web, web_theme_name);
    let expected = web_find_highlighted_listbox_option_chrome(theme, web_option_slot);

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

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        true,
        |cx| vec![build(cx, &open)],
    );

    let (snap, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ComboBox && n.label.as_deref() == Some(a11y_label))
        .expect("trigger semantics (combobox) by a11y label");
    ui.set_focus(Some(trigger.id));
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(2),
        true,
        |cx| vec![build(cx, &open)],
    );

    dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);

    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for tick in 0..settle_frames {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(3 + tick),
            tick + 1 == settle_frames,
            |cx| vec![build(cx, &open)],
        );
    }

    let (mut snap, mut scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    if fret_find_active_listbox_option(&snap).is_none() {
        // If the trigger key path did not produce an active item (some pages open via click and
        // move focus into an inner text field), force the open state and drive ArrowDown on the
        // first text field inside the overlay.
        let _ = app.models_mut().update(&open, |v| *v = true);
        let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
        for tick in 0..settle_frames {
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(3 + settle_frames + tick),
                tick + 1 == settle_frames,
                |cx| vec![build(cx, &open)],
            );
        }

        let (snap2, _) = paint_frame(&mut ui, &mut app, &mut services, bounds);
        if let Some(text_field) = snap2
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::TextField)
            .max_by(|a, b| rect_area(a.bounds).total_cmp(&rect_area(b.bounds)))
        {
            ui.set_focus(Some(text_field.id));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                FrameId(3 + settle_frames + settle_frames),
                true,
                |cx| vec![build(cx, &open)],
            );
            dispatch_key_press(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
        }

        (snap, scene) = paint_frame(&mut ui, &mut app, &mut services, bounds);
    }

    let option = fret_find_active_listbox_option(&snap).unwrap_or_else(|| {
        let focused_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.flags.focused)
            .map(|n| n.role)
            .collect();
        let listbox_count = snap.nodes.iter().filter(|n| n.role == SemanticsRole::ListBox).count();
        let option_count = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::ListBoxOption)
            .count();
        let active_owner_roles: Vec<SemanticsRole> = snap
            .nodes
            .iter()
            .filter(|n| n.active_descendant.is_some())
            .map(|n| n.role)
            .collect();
        panic!(
            "expected focused listbox option semantics node (or any active_descendant -> option)\n  listbox_count={listbox_count}\n  option_count={option_count}\n  focused_roles={focused_roles:?}\n  active_descendant_owner_roles={active_owner_roles:?}"
        )
    });

    let quad = find_best_solid_quad_within_matching_bg(&scene, option.bounds, expected.bg)
        .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option background quad"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option background"),
        color_to_rgba(quad.background),
        expected.bg,
        0.03,
    );

    let text = find_best_text_color_near(
        &scene,
        option.bounds,
        leftish_text_probe_point(option.bounds),
    )
    .unwrap_or_else(|| panic!("{web_name} {web_theme_name}: focused option text color"));
    assert_rgba_close(
        &format!("{web_name} {web_theme_name} focused option text color"),
        text,
        expected.fg,
        0.03,
    );
}
