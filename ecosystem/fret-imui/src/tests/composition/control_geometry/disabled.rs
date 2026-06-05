use super::*;

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
