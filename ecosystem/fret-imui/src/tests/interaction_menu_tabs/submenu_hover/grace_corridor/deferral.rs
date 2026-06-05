use super::*;

use super::geometry::find_grace_corridor_transition_points;
use super::timers::pending_nonrepeating_timer_tokens_after;

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
