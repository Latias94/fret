use super::*;

use super::geometry::find_safe_hover_corridor_points;
use super::timers::pending_nonrepeating_timer_tokens_after;

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
