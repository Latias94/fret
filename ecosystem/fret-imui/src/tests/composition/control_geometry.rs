use super::*;

fn center_of_rect(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0 * 0.5),
        Px(rect.origin.y.0 + rect.size.height.0 * 0.5),
    )
}

fn baseline_bounds(baseline: &[(&str, Rect)], test_id: &str) -> Rect {
    baseline
        .iter()
        .find_map(|(id, rect)| (*id == test_id).then_some(*rect))
        .unwrap_or_else(|| panic!("missing baseline bounds for {test_id}"))
}

fn control_bounds_for_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> Rect {
    let node = node_for_test_id(ui, app, services, bounds, test_id);
    ui.debug_node_bounds(node)
        .unwrap_or_else(|| panic!("missing layout bounds for {test_id}"))
}

fn assert_same_rect(test_id: &str, before: Rect, after: Rect, state: &str) {
    assert_eq!(
        after.origin, before.origin,
        "{test_id} origin changed during {state}"
    );
    assert_eq!(
        after.size, before.size,
        "{test_id} size changed during {state}"
    );
}

#[test]
fn button_family_variants_and_radio_mount_with_expected_bounds() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-family-variants",
        |cx| {
            crate::imui_raw(cx, |ui| {
                use fret_ui_kit::imui::{
                    ButtonArrowDirection, ButtonOptions, RadioOptions, UiWriterImUiFacadeExt as _,
                };

                let _ = ui.small_button_with_options(
                    "Quick save",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-variants.small")),
                        ..Default::default()
                    },
                );
                let _ = ui.arrow_button_with_options(
                    "imui-variants.arrow.left",
                    ButtonArrowDirection::Left,
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-variants.arrow.left")),
                        ..Default::default()
                    },
                );
                let _ = ui.invisible_button_with_options(
                    "imui-variants.hotspot",
                    Size::new(Px(48.0), Px(24.0)),
                    ButtonOptions {
                        a11y_label: Some(Arc::from("Timeline hotspot")),
                        test_id: Some(Arc::from("imui-variants.hotspot")),
                        ..Default::default()
                    },
                );
                let _ = ui.radio_with_options(
                    "Move tool",
                    true,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-variants.radio")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.small",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.arrow.left",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.hotspot",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-variants.radio",
    ));

    let arrow_bounds = bounds_for_test_id(&ui, "imui-variants.arrow.left");
    assert_eq!(arrow_bounds.size.width, arrow_bounds.size.height);

    let hotspot_bounds = bounds_for_test_id(&ui, "imui-variants.hotspot");
    assert_eq!(hotspot_bounds.size.width, Px(48.0));
    assert_eq!(hotspot_bounds.size.height, Px(24.0));
}

#[test]
fn base_control_state_changes_keep_outer_bounds_stable() {
    use fret_ui_kit::imui::{ButtonOptions, RadioOptions};

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(360.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let checkbox = app.models_mut().insert(false);
    let switch = app.models_mut().insert(false);
    let slider = app.models_mut().insert(0.25f32);
    let radio_selected = Rc::new(Cell::new(false));
    let selectable_selected = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.button_with_options(
                    "Button",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-geometry.button")),
                        ..Default::default()
                    },
                );
                let _ = ui.checkbox_model_with_options(
                    "Checkbox",
                    &checkbox,
                    CheckboxOptions {
                        test_id: Some(Arc::from("imui-geometry.checkbox")),
                        ..Default::default()
                    },
                );
                let _ = ui.radio_with_options(
                    "Radio",
                    radio_selected.get(),
                    RadioOptions {
                        test_id: Some(Arc::from("imui-geometry.radio")),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Switch",
                    &switch,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-geometry.switch")),
                        ..Default::default()
                    },
                );
                let _ = ui.slider_f32_model_with_options(
                    "Slider",
                    &slider,
                    SliderOptions {
                        test_id: Some(Arc::from("imui-geometry.slider")),
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                        ..Default::default()
                    },
                );
                let _ = ui.combo_with_options(
                    "geometry-combo",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-geometry.combo")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-geometry.combo.alpha")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let _ = ui.selectable_with_options(
                    "Selectable",
                    SelectableOptions {
                        selected: selectable_selected.get(),
                        test_id: Some(Arc::from("imui-geometry.selectable")),
                        ..Default::default()
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
        "imui-base-control-geometry",
        |cx| render(cx),
    );

    let control_ids = [
        "imui-geometry.button",
        "imui-geometry.checkbox",
        "imui-geometry.radio",
        "imui-geometry.switch",
        "imui-geometry.slider",
        "imui-geometry.combo",
        "imui-geometry.selectable",
    ];
    let mut baseline = Vec::new();
    for test_id in control_ids {
        baseline.push((
            test_id,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
        ));
    }

    for test_id in control_ids {
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
            "imui-base-control-geometry",
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
            "imui-base-control-geometry",
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
            "imui-base-control-geometry",
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

    for test_id in [
        "imui-geometry.checkbox",
        "imui-geometry.switch",
        "imui-geometry.slider",
        "imui-geometry.combo",
    ] {
        let before = baseline_bounds(&baseline, test_id);
        click_at(&mut ui, &mut app, &mut services, center_of_rect(before));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-base-control-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "value/open",
        );
    }

    radio_selected.set(true);
    selectable_selected.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-base-control-geometry",
        &render,
    );
    for test_id in ["imui-geometry.radio", "imui-geometry.selectable"] {
        let before = baseline_bounds(&baseline, test_id);
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "selected",
        );
    }
}

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

#[test]
fn control_disabled_state_changes_keep_outer_bounds_stable() {
    use fret_ui_kit::imui::{
        BeginMenuOptions, BeginSubmenuOptions, ButtonOptions, RadioOptions, TabBarOptions,
        TabItemOptions, TextAreaOptions,
    };

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(520.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let disabled = Rc::new(Cell::new(false));
    let input = app.models_mut().insert(String::from("Input"));
    let textarea = app.models_mut().insert(String::from("Textarea"));
    let checkbox = app.models_mut().insert(false);
    let switch = app.models_mut().insert(false);
    let slider = app.models_mut().insert(0.25f32);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let enabled = !disabled.get();
            ui.vertical(|ui| {
                let _ = ui.input_text_model_with_options(
                    &input,
                    InputTextOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.input")),
                        ..Default::default()
                    },
                );
                let _ = ui.textarea_model_with_options(
                    &textarea,
                    TextAreaOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.textarea")),
                        ..Default::default()
                    },
                );
                let _ = ui.button_with_options(
                    "Button",
                    ButtonOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.button")),
                        ..Default::default()
                    },
                );
                let _ = ui.checkbox_model_with_options(
                    "Checkbox",
                    &checkbox,
                    CheckboxOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.checkbox")),
                        ..Default::default()
                    },
                );
                let _ = ui.radio_with_options(
                    "Radio",
                    false,
                    RadioOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.radio")),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Switch",
                    &switch,
                    SwitchOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.switch")),
                        ..Default::default()
                    },
                );
                let _ = ui.slider_f32_model_with_options(
                    "Slider",
                    &slider,
                    SliderOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.slider")),
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                        ..Default::default()
                    },
                );
                let _ = ui.combo_with_options(
                    "disabled-combo",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.combo")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-disabled.combo.alpha")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let _ = ui.selectable_with_options(
                    "Selectable",
                    SelectableOptions {
                        enabled,
                        test_id: Some(Arc::from("imui-disabled.selectable")),
                        ..Default::default()
                    },
                );

                ui.menu_bar(|ui| {
                    let _ = ui.begin_menu_with_options(
                        "file-disabled",
                        "File",
                        BeginMenuOptions {
                            enabled,
                            test_id: Some(Arc::from("imui-disabled.menu.file")),
                            ..Default::default()
                        },
                        |_ui| {},
                    );
                    let _ = ui.begin_menu_with_options(
                        "more-disabled",
                        "More",
                        BeginMenuOptions {
                            test_id: Some(Arc::from("imui-disabled.menu.more")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent-disabled",
                                "Recent",
                                BeginSubmenuOptions {
                                    enabled,
                                    test_id: Some(Arc::from("imui-disabled.menu.more.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-disabled.menu.more.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                });

                ui.tab_bar_with_options(
                    "disabled-tabs",
                    TabBarOptions {
                        test_id: Some(Arc::from("imui-disabled.tabs.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene",
                            TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-disabled.tabs.scene")),
                                panel_test_id: Some(Arc::from("imui-disabled.tabs.scene.panel")),
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
                                enabled,
                                test_id: Some(Arc::from("imui-disabled.tabs.inspector")),
                                panel_test_id: Some(Arc::from(
                                    "imui-disabled.tabs.inspector.panel",
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
        "imui-disabled-geometry",
        |cx| render(cx),
    );

    let disabled_ids = [
        "imui-disabled.input",
        "imui-disabled.textarea",
        "imui-disabled.button",
        "imui-disabled.checkbox",
        "imui-disabled.radio",
        "imui-disabled.switch",
        "imui-disabled.slider",
        "imui-disabled.combo",
        "imui-disabled.selectable",
        "imui-disabled.menu.file",
        "imui-disabled.tabs.inspector",
    ];
    let mut baseline = Vec::new();
    for test_id in disabled_ids {
        baseline.push((
            test_id,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
        ));
    }

    let more_before = control_bounds_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-disabled.menu.more",
    );
    click_at(
        &mut ui,
        &mut app,
        &mut services,
        center_of_rect(more_before),
    );
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-disabled-geometry",
        &render,
    );
    let submenu_test_id = "imui-disabled.menu.more.recent";
    let submenu_before =
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id);

    disabled.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-disabled-geometry",
        &render,
    );

    for test_id in disabled_ids {
        let before = baseline_bounds(&baseline, test_id);
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "disabled",
        );
    }
    assert_same_rect(
        submenu_test_id,
        submenu_before,
        control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, submenu_test_id),
        "disabled",
    );
}
