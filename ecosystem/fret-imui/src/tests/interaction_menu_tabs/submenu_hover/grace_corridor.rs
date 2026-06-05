use super::*;

use fret_ui_kit::primitives::menu::pointer_grace_intent;

fn find_grace_corridor_transition_points(
    reference: Rect,
    sibling: Rect,
    floating: Rect,
) -> Option<(Point, Point)> {
    let geometry = pointer_grace_intent::PointerGraceIntentGeometry {
        reference,
        floating,
    };
    let reference_right = reference.origin.x.0 + reference.size.width.0;
    let sibling_right = sibling.origin.x.0 + sibling.size.width.0;
    let sibling_bottom = sibling.origin.y.0 + sibling.size.height.0;

    for exit_y in (reference.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
        for exit_x in
            (reference.origin.x.0.floor() as i32)..=((reference_right + 24.0).ceil() as i32)
        {
            let exit = Point::new(Px(exit_x as f32), Px(exit_y as f32));
            if reference.contains(exit) || sibling.contains(exit) || floating.contains(exit) {
                continue;
            }

            let Some(intent) =
                pointer_grace_intent::grace_intent_from_exit_point(exit, geometry, Px(5.0))
            else {
                continue;
            };

            for y in (sibling.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
                for x in (sibling.origin.x.0.floor() as i32)..=(sibling_right.ceil() as i32) {
                    let candidate = Point::new(Px(x as f32), Px(y as f32));
                    if !sibling.contains(candidate) {
                        continue;
                    }

                    let moving_towards = match intent.side {
                        pointer_grace_intent::GraceSide::Right => candidate.x.0 > exit.x.0,
                        pointer_grace_intent::GraceSide::Left => candidate.x.0 < exit.x.0,
                    };
                    if moving_towards
                        && pointer_grace_intent::is_pointer_in_grace_area(candidate, intent)
                    {
                        return Some((exit, candidate));
                    }
                }
            }
        }
    }

    None
}

fn pending_nonrepeating_timer_tokens_after(
    app: &TestHost,
    after: std::time::Duration,
) -> Vec<TimerToken> {
    app.effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                token,
                after: effect_after,
                repeat,
                ..
            } if repeat.is_none() && *effect_after == after => Some(*token),
            _ => None,
        })
        .collect()
}

fn find_safe_hover_corridor_points(
    search_bounds: Rect,
    reference: Rect,
    floating: Rect,
    buffer: Px,
) -> Option<(Point, Point)> {
    let geometry = pointer_grace_intent::PointerGraceIntentGeometry {
        reference,
        floating,
    };

    let mut safe_point: Option<Point> = None;
    for y in (0..=search_bounds.size.height.0 as i32).step_by(2) {
        for x in (0..=search_bounds.size.width.0 as i32).step_by(2) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            if pos.x.0 <= reference.origin.x.0 + reference.size.width.0 {
                continue;
            }
            if reference.contains(pos) || floating.contains(pos) {
                continue;
            }
            if !pointer_grace_intent::last_pointer_is_safe(pos, geometry, buffer) {
                continue;
            }
            safe_point = Some(pos);
            break;
        }
        if safe_point.is_some() {
            break;
        }
    }

    let safe_point = safe_point?;
    let mut unsafe_point: Option<Point> = None;
    for y in (0..=search_bounds.size.height.0 as i32).step_by(4) {
        for x in (0..=search_bounds.size.width.0 as i32).step_by(4) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            if pos.x.0 >= safe_point.x.0 {
                continue;
            }
            if reference.contains(pos) || floating.contains(pos) {
                continue;
            }
            if pointer_grace_intent::last_pointer_is_safe(pos, geometry, buffer) {
                continue;
            }
            unsafe_point = Some(pos);
            break;
        }
        if unsafe_point.is_some() {
            break;
        }
    }

    unsafe_point.map(|unsafe_point| (unsafe_point, safe_point))
}

#[test]
fn begin_submenu_helper_defers_sibling_switch_inside_grace_corridor() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let history_hovered = Rc::new(Cell::new(false));
    let history_hovered_raw = Rc::new(Cell::new(false));
    let history_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let history_hovered = history_hovered.clone();
        let history_hovered_raw = history_hovered_raw.clone();
        let history_hovered_raw_below_barrier = history_hovered_raw_below_barrier.clone();
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-grace-corridor.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-grace-corridor.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let history = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            history_hovered.set(history.response().hovered());
                            history_hovered_raw.set(history.response().pointer_hovered_raw());
                            history_hovered_raw_below_barrier
                                .set(history.response().pointer_hovered_raw_below_barrier());
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let recent_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.recent");
    let history_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.history");
    let recent_popup_bounds = bounds_for_test_id(&ui, "imui-popup-recent");
    let (grace_exit_point, history_grace_point) =
        find_grace_corridor_transition_points(recent_bounds, history_bounds, recent_popup_bounds)
            .expect("expected a history point inside the submenu grace corridor");

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        grace_exit_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        history_grace_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    assert!(
        history_hovered.get()
            || history_hovered_raw.get()
            || history_hovered_raw_below_barrier.get(),
        "expected pointer to hit the sibling trigger inside grace corridor (exit={grace_exit_point:?} hit={history_grace_point:?})"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let grace_timeout =
        fret_ui_kit::primitives::menu::sub::MenuSubmenuConfig::default().pointer_grace_timeout;
    let pending = pending_nonrepeating_timer_tokens_after(&app, grace_timeout);
    let dispatched = dispatch_timer_tokens(&mut ui, &mut app, &mut services, &pending);
    assert!(
        dispatched > 0,
        "expected pointer grace timeout timer to be present"
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));
}

#[test]
fn begin_submenu_helper_safe_corridor_cancels_close_timer() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-safe-corridor.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-safe-corridor.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-safe-corridor.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-safe-corridor.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Other",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-safe-corridor.file.other",
                                    )),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent.project",
    ));

    let submenu_cfg = fret_ui_kit::primitives::menu::sub::MenuSubmenuConfig::default();
    let recent_bounds = bounds_for_test_id(&ui, "imui-submenu-safe-corridor.file.recent");
    let recent_popup_bounds = bounds_for_test_id(&ui, "imui-popup-recent");
    let (unsafe_point, safe_point) = find_safe_hover_corridor_points(
        bounds,
        recent_bounds,
        recent_popup_bounds,
        submenu_cfg.safe_hover_buffer,
    )
    .expect("expected safe/unsafe corridor points around the open submenu");

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        unsafe_point,
        MouseButtons::default(),
    );
    let close_tokens = pending_nonrepeating_timer_tokens_after(&app, submenu_cfg.close_delay);
    assert!(
        !close_tokens.is_empty(),
        "expected unsafe pointer move to arm a close-delay timer (unsafe_point={unsafe_point:?})"
    );
    let close_token = close_tokens[0];

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        safe_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    assert!(
        app.effects
            .iter()
            .any(|effect| matches!(effect, Effect::CancelTimer { token } if *token == close_token)),
        "expected safe corridor pointer move to cancel the close-delay timer (safe_point={safe_point:?} close_token={close_token:?} effects={:?})",
        app.effects
    );
    assert!(
        pending_nonrepeating_timer_tokens_after(&app, submenu_cfg.close_delay).is_empty(),
        "expected safe corridor pointer move to avoid arming a new close-delay timer"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent.project",
    ));
}
