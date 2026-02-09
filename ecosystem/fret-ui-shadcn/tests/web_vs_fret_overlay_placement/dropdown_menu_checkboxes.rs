use super::*;

#[test]
fn web_vs_fret_dropdown_menu_checkboxes_overlay_placement_matches() {
    assert_overlay_placement_matches(
        "dropdown-menu-checkboxes",
        Some("menu"),
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
                };

            build_dropdown_menu_checkboxes_demo(
                cx,
                open,
                checked_status_bar,
                checked_activity_bar,
                checked_panel,
            )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_overlay_placement_matches_mobile_tiny_viewport() {
    assert_overlay_placement_matches(
        "dropdown-menu-checkboxes.vp375x240",
        Some("menu"),
        |cx, open| {
            #[derive(Default)]
            struct Models {
                checked_status_bar: Option<Model<bool>>,
                checked_activity_bar: Option<Model<bool>>,
                checked_panel: Option<Model<bool>>,
            }

            let existing = cx.with_state(Models::default, |st| {
                match (
                    st.checked_status_bar.as_ref(),
                    st.checked_activity_bar.as_ref(),
                    st.checked_panel.as_ref(),
                ) {
                    (Some(a), Some(b), Some(c)) => Some((a.clone(), b.clone(), c.clone())),
                    _ => None,
                }
            });

            let (checked_status_bar, checked_activity_bar, checked_panel) =
                if let Some(existing) = existing {
                    existing
                } else {
                    let checked_status_bar = cx.app.models_mut().insert(true);
                    let checked_activity_bar = cx.app.models_mut().insert(false);
                    let checked_panel = cx.app.models_mut().insert(false);

                    cx.with_state(Models::default, |st| {
                        st.checked_status_bar = Some(checked_status_bar.clone());
                        st.checked_activity_bar = Some(checked_activity_bar.clone());
                        st.checked_panel = Some(checked_panel.clone());
                    });

                    (checked_status_bar, checked_activity_bar, checked_panel)
                };

            build_dropdown_menu_checkboxes_demo(
                cx,
                open,
                checked_status_bar,
                checked_activity_bar,
                checked_panel,
            )
        },
        SemanticsRole::Button,
        Some("Open"),
        SemanticsRole::Menu,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_checkbox_indicator_slot_inset_matches_web() {
    assert_dropdown_menu_checkboxes_indicator_slot_inset_matches_web_impl(
        "dropdown-menu-checkboxes",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_checkbox_indicator_slot_inset_matches_web_mobile_tiny_viewport()
 {
    assert_dropdown_menu_checkboxes_indicator_slot_inset_matches_web_impl(
        "dropdown-menu-checkboxes.vp375x240",
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_content_insets_match() {
    let web_name = "dropdown-menu-checkboxes";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_content_insets_match_mobile_tiny_viewport() {
    let web_name = "dropdown-menu-checkboxes.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);

    let expected = web_menu_content_insets_for_slots(&theme, &["dropdown-menu-content"]);
    let expected_menu_h = web_portal_node_by_data_slot(&theme, "dropdown-menu-content")
        .rect
        .h;

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual = fret_menu_content_insets(&snap);
    assert_sorted_insets_match(web_name, &actual, &expected);
    let actual_menu_h = fret_largest_menu_height(&snap)
        .unwrap_or_else(|| panic!("missing fret menu for {web_name}"));
    assert_close(
        &format!("{web_name} menu height"),
        actual_menu_h,
        expected_menu_h,
        2.0,
    );
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_item_height_matches() {
    let web_name = "dropdown-menu-checkboxes";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-checkbox-item"]);
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web dropdown-menu-checkbox-item height for {web_name}")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
#[test]
fn web_vs_fret_dropdown_menu_checkboxes_menu_item_height_matches_mobile_tiny_viewport() {
    let web_name = "dropdown-menu-checkboxes.vp375x240";
    let web = read_web_golden_open(web_name);
    let theme = web_theme(&web);
    let expected_hs = web_portal_slot_heights(&theme, &["dropdown-menu-checkbox-item"]);
    let expected_h =
        expected_hs.iter().copied().next().unwrap_or_else(|| {
            panic!("missing web dropdown-menu-checkbox-item height for {web_name}")
        });

    let window = AppWindowId::default();
    let mut app = App::new();
    setup_app_with_shadcn_theme(&mut app);

    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = StyleAwareServices::default();

    let bounds = bounds_for_web_theme(&theme);
    let open: Model<bool> = app.models_mut().insert(false);
    let checked_status_bar: Model<bool> = app.models_mut().insert(true);
    let checked_activity_bar: Model<bool> = app.models_mut().insert(false);
    let checked_panel: Model<bool> = app.models_mut().insert(false);

    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        FrameId(1),
        false,
        |cx| {
            let el = build_dropdown_menu_checkboxes_demo(
                cx,
                &open,
                checked_status_bar.clone(),
                checked_activity_bar.clone(),
                checked_panel.clone(),
            );
            vec![pad_root(cx, Px(0.0), el)]
        },
    );
    let _ = app.models_mut().update(&open, |v| *v = true);
    let settle_frames = fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2;
    for frame in 2..=(2 + settle_frames) {
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            FrameId(frame),
            frame == 2 + settle_frames,
            |cx| {
                let el = build_dropdown_menu_checkboxes_demo(
                    cx,
                    &open,
                    checked_status_bar.clone(),
                    checked_activity_bar.clone(),
                    checked_panel.clone(),
                );
                vec![pad_root(cx, Px(0.0), el)]
            },
        );
    }

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let actual_hs = fret_menu_item_heights_in_menus(&snap);
    assert_menu_item_row_height_matches(web_name, expected_h.round(), &actual_hs, 1.0);
}
