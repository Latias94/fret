use super::*;

#[test]
fn menu_and_tab_trigger_state_changes_keep_outer_bounds_stable() {
    use fret_ui_kit::imui::{
        BeginMenuOptions, BeginSubmenuOptions, MenuBarOptions, TabBarOptions, TabItemOptions,
    };

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(320.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                ui.menu_bar_with_options(
                    MenuBarOptions {
                        test_id: Some(Arc::from("imui-geometry.menu.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "file",
                            "File",
                            BeginMenuOptions {
                                test_id: Some(Arc::from("imui-geometry.menu.file")),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.begin_submenu_with_options(
                                    "recent",
                                    "Recent",
                                    BeginSubmenuOptions {
                                        test_id: Some(Arc::from("imui-geometry.menu.file.recent")),
                                        ..Default::default()
                                    },
                                    |ui| {
                                        let _ = ui.menu_item_with_options(
                                            "Project",
                                            MenuItemOptions {
                                                test_id: Some(Arc::from(
                                                    "imui-geometry.menu.file.recent.project",
                                                )),
                                                ..Default::default()
                                            },
                                        );
                                    },
                                );
                                let _ = ui.menu_item_with_options(
                                    "Open",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-geometry.menu.file.open")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );
                    },
                );

                ui.tab_bar_with_options(
                    "geometry-tabs",
                    TabBarOptions {
                        test_id: Some(Arc::from("imui-geometry.tabs.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene",
                            TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-geometry.tabs.scene")),
                                panel_test_id: Some(Arc::from("imui-geometry.tabs.scene.panel")),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Scene Panel");
                            },
                        );
                        tabs.begin_tab_item_with_options(
                            "inspector",
                            "Inspector",
                            TabItemOptions {
                                test_id: Some(Arc::from("imui-geometry.tabs.inspector")),
                                panel_test_id: Some(Arc::from(
                                    "imui-geometry.tabs.inspector.panel",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Inspector Panel");
                            },
                        );
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        |cx| render(cx),
    );

    let top_level_ids = [
        "imui-geometry.menu.file",
        "imui-geometry.tabs.scene",
        "imui-geometry.tabs.inspector",
    ];
    let mut baseline = Vec::new();
    for test_id in top_level_ids {
        baseline.push((
            test_id,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
        ));
    }

    for test_id in top_level_ids {
        let before = baseline_bounds(&baseline, test_id);
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            center_of_rect(before),
            MouseButtons::default(),
        );
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-menu-tab-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "hover",
        );

        let node = node_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id);
        ui.set_focus(Some(node));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-menu-tab-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "focus",
        );

        pointer_down_at(&mut ui, &mut app, &mut services, center_of_rect(before));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-menu-tab-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "pressed",
        );
        pointer_up_at_with_is_click(
            &mut ui,
            &mut app,
            &mut services,
            center_of_rect(before),
            false,
        );
    }

    let inspector_before = baseline_bounds(&baseline, "imui-geometry.tabs.inspector");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(inspector_before),
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    for test_id in ["imui-geometry.tabs.scene", "imui-geometry.tabs.inspector"] {
        let before = baseline_bounds(&baseline, test_id);
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "selected",
        );
    }

    let file_before = baseline_bounds(&baseline, "imui-geometry.menu.file");
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(file_before),
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    assert_same_rect(
        "imui-geometry.menu.file",
        file_before,
        control_bounds_for_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-geometry.menu.file",
        ),
        "open",
    );

    let submenu_test_id = "imui-geometry.menu.file.recent";
    let submenu_before =
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id);

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(submenu_before),
        MouseButtons::default(),
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    assert_same_rect(
        submenu_test_id,
        submenu_before,
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id),
        "hover",
    );

    let submenu_node = node_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id);
    ui.set_focus(Some(submenu_node));
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    assert_same_rect(
        submenu_test_id,
        submenu_before,
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id),
        "focus",
    );

    pointer_down_at(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(submenu_before),
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    assert_same_rect(
        submenu_test_id,
        submenu_before,
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id),
        "pressed",
    );
    pointer_up_at_with_is_click(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(submenu_before),
        true,
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-tab-geometry",
        &render,
    );
    assert_same_rect(
        submenu_test_id,
        submenu_before,
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id),
        "open",
    );
}
