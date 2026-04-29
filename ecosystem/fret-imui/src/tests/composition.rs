use super::*;

use fret_ui::ScrollHandle;
use fret_ui_kit::imui::{ChildRegionChrome, ChildRegionOptions};

#[test]
fn ui_writer_imui_facade_ext_compiles() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-ui-writer-facade-ext",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui_writer_imui_facade_ext_smoke(ui);
            })
        },
    );

    assert_eq!(ui.children(root).len(), 3);
}

#[test]
fn ui_kit_builder_can_be_rendered_from_imui() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-ui-kit-bridge",
        |cx| {
            crate::imui_raw(cx, |ui| {
                use fret_ui_kit::imui::UiWriterUiKitExt as _;

                let builder = fret_ui_kit::ui::text("Hello").text_sm();
                ui.add_ui(builder);
            })
        },
    );

    assert_eq!(ui.children(root).len(), 1);
}

#[test]
fn imui_default_mounts_with_stacked_host() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-default-stacked-host",
        |cx| {
            crate::imui(cx, |ui| {
                ui.menu_item_with_options(
                    "First",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-default.first")),
                        ..Default::default()
                    },
                );
                ui.menu_item_with_options(
                    "Second",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-default.second")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert_eq!(ui.children(root).len(), 1);

    let host = ui.children(root)[0];
    assert_eq!(ui.children(host).len(), 2);

    let host_bounds = ui.debug_node_bounds(host).expect("host bounds");
    assert_eq!(host_bounds.size.width, bounds.size.width);
    assert_eq!(host_bounds.size.height, bounds.size.height);

    let first = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-default.first",
    );
    let second = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-default.second",
    );
    assert!(second.y.0 > first.y.0);
}

#[test]
fn imui_raw_preserves_direct_sibling_emission() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-raw-direct-siblings",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "First",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-raw.first")),
                        ..Default::default()
                    },
                );
                ui.menu_item_with_options(
                    "Second",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-raw.second")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert_eq!(ui.children(root).len(), 2);
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-raw.first",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-raw.second",
    ));
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
fn container_helpers_layout_horizontal_vertical_grid_and_scroll() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    fret_ui::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = fret_ui::theme::ThemeConfig {
            name: "Test".to_string(),
            ..fret_ui::theme::ThemeConfig::default()
        };
        cfg.colors.insert(
            "scrollbar.track.background".to_string(),
            "#1f1f1f".to_string(),
        );
        cfg.colors.insert(
            "scrollbar.thumb.background".to_string(),
            "#5f5f5f".to_string(),
        );
        cfg.colors.insert(
            "scrollbar.thumb.hover.background".to_string(),
            "#7f7f7f".to_string(),
        );
        cfg.metrics
            .insert("metric.scrollbar.width".to_string(), 8.0);
        theme.apply_config_patch(&cfg);
    });
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-container-helpers-layout",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.vertical_with_options(
                    VerticalOptions {
                        gap: Px(8.0).into(),
                        ..Default::default()
                    },
                    |ui| {
                        ui.horizontal_with_options(
                            HorizontalOptions {
                                gap: Px(10.0).into(),
                                ..Default::default()
                            },
                            |ui| {
                                ui.menu_item_with_options(
                                    "Left",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-container-left")),
                                        ..Default::default()
                                    },
                                );
                                ui.menu_item_with_options(
                                    "Right",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-container-right")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );

                        ui.grid_with_options(
                            GridOptions {
                                columns: 2,
                                column_gap: Px(6.0).into(),
                                row_gap: Px(6.0).into(),
                                ..Default::default()
                            },
                            |ui| {
                                ui.menu_item_with_options(
                                    "A",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-grid-a")),
                                        ..Default::default()
                                    },
                                );
                                ui.menu_item_with_options(
                                    "B",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-grid-b")),
                                        ..Default::default()
                                    },
                                );
                                ui.menu_item_with_options(
                                    "C",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-grid-c")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );

                        ui.scroll_with_options(
                            ScrollOptions {
                                axis: fret_ui::element::ScrollAxis::X,
                                show_scrollbar_x: true,
                                show_scrollbar_y: false,
                                ..Default::default()
                            },
                            |ui| {
                                ui.menu_item_with_options(
                                    "Scroll Child",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-scroll-child")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );
                    },
                );
            })
        },
    );

    let left = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-container-left",
    );
    let right = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-container-right",
    );
    assert!(right.x.0 > left.x.0);

    let grid_a = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-a");
    let grid_b = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-b");
    let grid_c = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-grid-c");
    assert!(grid_b.x.0 > grid_a.x.0);
    assert!(grid_c.y.0 > grid_a.y.0);

    let scroll_child = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-scroll-child",
    );
    assert!(scroll_child.y.0 > grid_c.y.0);
}

#[test]
fn menu_bar_helper_arranges_triggers_horizontally_and_stamps_menubar_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(140.0)),
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
        "imui-menu-bar",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_bar_with_options(
                    fret_ui_kit::imui::MenuBarOptions {
                        test_id: Some(Arc::from("imui-menu-bar.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "file",
                            "File",
                            fret_ui_kit::imui::BeginMenuOptions {
                                test_id: Some(Arc::from("imui-menu-bar.file")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                        let _ = ui.begin_menu_with_options(
                            "edit",
                            "Edit",
                            fret_ui_kit::imui::BeginMenuOptions {
                                test_id: Some(Arc::from("imui-menu-bar.edit")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let file = bounds_for_test_id(&ui, "imui-menu-bar.file");
    let edit = bounds_for_test_id(&ui, "imui-menu-bar.edit");
    assert!(edit.origin.x.0 > file.origin.x.0 + file.size.width.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let menubar = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-menu-bar.root"))
        .expect("menubar semantics node");
    assert_eq!(menubar.role, SemanticsRole::MenuBar);
}

#[test]
fn tab_bar_helper_arranges_tabs_horizontally_and_stamps_tab_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
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
        "imui-tab-bar",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.tab_bar_with_options(
                    "workspace",
                    fret_ui_kit::imui::TabBarOptions {
                        test_id: Some(Arc::from("imui-tab-bar.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene",
                            fret_ui_kit::imui::TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-tab-bar.scene")),
                                panel_test_id: Some(Arc::from("imui-tab-bar.scene.panel")),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Scene Panel");
                            },
                        );
                        tabs.begin_tab_item_with_options(
                            "inspector",
                            "Inspector",
                            fret_ui_kit::imui::TabItemOptions {
                                test_id: Some(Arc::from("imui-tab-bar.inspector")),
                                panel_test_id: Some(Arc::from("imui-tab-bar.inspector.panel")),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Inspector Panel");
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let scene = bounds_for_test_id(&ui, "imui-tab-bar.scene");
    let inspector = bounds_for_test_id(&ui, "imui-tab-bar.inspector");
    assert!(inspector.origin.x.0 > scene.origin.x.0 + scene.size.width.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let tab_list = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.root"))
        .expect("tab list semantics node");
    assert_eq!(tab_list.role, SemanticsRole::TabList);

    let scene_tab = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.scene"))
        .expect("scene tab semantics node");
    assert_eq!(scene_tab.role, SemanticsRole::Tab);
    assert!(scene_tab.flags.selected);

    let scene_panel = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar.scene.panel"))
        .expect("scene panel semantics node");
    assert_eq!(scene_panel.role, SemanticsRole::TabPanel);
    assert_eq!(scene_panel.label.as_deref(), Some("Scene"));

    assert!(
        snap.nodes
            .iter()
            .all(|node| node.test_id.as_deref() != Some("imui-tab-bar.inspector.panel")),
        "expected inactive tab panel to stay out of the semantics tree"
    );
}

#[test]
fn child_region_helper_stacks_content_and_forwards_scroll_options() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let handle = ScrollHandle::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.child_region_with_options(
                "imui-child-region",
                ChildRegionOptions {
                    layout: fret_ui_kit::LayoutRefinement::default(),
                    scroll: fret_ui_kit::imui::ScrollOptions {
                        handle: Some(handle.clone()),
                        viewport_test_id: Some(Arc::from("imui-child-region.viewport")),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-child-region")),
                    content_test_id: Some(Arc::from("imui-child-region.content")),
                    ..Default::default()
                },
                |ui| {
                    for index in 0..24 {
                        ui.menu_item_with_options(
                            format!("Row {index}"),
                            fret_ui_kit::imui::MenuItemOptions {
                                test_id: Some(Arc::from(format!("imui-child-region.row.{index}"))),
                                ..Default::default()
                            },
                        );
                    }
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
        "imui-child-region",
        render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.viewport",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.content",
    ));

    let row0 = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.row.0",
    );
    let row1 = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.row.1",
    );
    assert!(row1.y.0 > row0.y.0);

    handle.set_offset(Point::new(Px(0.0), Px(80.0)));
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region",
        render,
    );

    assert!(handle.offset().y.0 > 0.0);
}

#[test]
fn child_region_helper_can_host_menu_bar_and_popup_menu() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.child_region_with_options(
                "imui-child-region-with-menu",
                ChildRegionOptions {
                    test_id: Some(Arc::from("imui-child-region-with-menu")),
                    content_test_id: Some(Arc::from("imui-child-region-with-menu.content")),
                    ..Default::default()
                },
                |ui| {
                    ui.menu_bar_with_options(
                        fret_ui_kit::imui::MenuBarOptions {
                            test_id: Some(Arc::from("imui-child-region-with-menu.menubar")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_menu_with_options(
                                "file",
                                "File",
                                fret_ui_kit::imui::BeginMenuOptions {
                                    test_id: Some(Arc::from("imui-child-region-with-menu.file")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Open",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-child-region-with-menu.file.open",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                    ui.menu_item_with_options(
                        "Body row",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-child-region-with-menu.body")),
                            ..Default::default()
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
        "imui-child-region-with-menu",
        render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.content",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.menubar",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.body",
    ));

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region-with-menu",
        &render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.file.open",
    ));
}

#[test]
fn child_region_helper_can_switch_between_framed_and_bare_chrome() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.horizontal_with_options(
                HorizontalOptions {
                    gap: fret_ui_kit::MetricRef::space(fret_ui_kit::Space::N2),
                    ..Default::default()
                },
                |ui| {
                    ui.child_region_with_options(
                        "imui-child-region.chrome.framed",
                        ChildRegionOptions {
                            layout: fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(148.0))
                                .h_px(Px(84.0)),
                            test_id: Some(Arc::from("imui-child-region.chrome.framed")),
                            content_test_id: Some(Arc::from(
                                "imui-child-region.chrome.framed.content",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_with_options(
                                "Framed",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-child-region.chrome.framed.row")),
                                    ..Default::default()
                                },
                            );
                        },
                    );

                    ui.child_region_with_options(
                        "imui-child-region.chrome.bare",
                        ChildRegionOptions {
                            chrome: ChildRegionChrome::Bare,
                            layout: fret_ui_kit::LayoutRefinement::default()
                                .w_px(Px(148.0))
                                .h_px(Px(84.0)),
                            test_id: Some(Arc::from("imui-child-region.chrome.bare")),
                            content_test_id: Some(Arc::from(
                                "imui-child-region.chrome.bare.content",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.menu_item_with_options(
                                "Bare",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-child-region.chrome.bare.row")),
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
        "imui-child-region-chrome",
        render,
    );

    let framed_region = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.chrome.framed",
    );
    let bare_region = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.chrome.bare",
    );
    let framed_bounds = ui.debug_node_bounds(framed_region).expect("framed bounds");
    let bare_bounds = ui.debug_node_bounds(bare_region).expect("bare bounds");
    let framed_row = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.chrome.framed.row",
    );
    let bare_row = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.chrome.bare.row",
    );

    let framed_dx = framed_row.x.0 - framed_bounds.origin.x.0;
    let bare_dx = bare_row.x.0 - bare_bounds.origin.x.0;
    let framed_dy = framed_row.y.0 - framed_bounds.origin.y.0;
    let bare_dy = bare_row.y.0 - bare_bounds.origin.y.0;

    assert!(framed_dx > bare_dx + 1.0);
    assert!(framed_dy > bare_dy + 1.0);
}

#[test]
fn table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)),
        TableColumn::px("Owner", Px(88.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-layout",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-layout",
                    &columns,
                    TableOptions {
                        striped: true,
                        test_id: Some(Arc::from("imui-table-layout")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text(
                                "Extremely long inspector label that should remain clipped inside the first fill column",
                            );
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                        table.row("beta", |row| {
                            row.cell_text("Short");
                            row.cell_text("Busy");
                            row.cell_text("Bob");
                        });
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let header_status = bounds_for_test_id(&ui, "imui-table-layout.header.cell.status");
    let row0_status = bounds_for_test_id(&ui, "imui-table-layout.row.0.cell.status");
    let row1_status = bounds_for_test_id(&ui, "imui-table-layout.row.1.cell.status");
    let header_owner = bounds_for_test_id(&ui, "imui-table-layout.header.cell.owner");
    let row0_owner = bounds_for_test_id(&ui, "imui-table-layout.row.0.cell.owner");
    let row1_owner = bounds_for_test_id(&ui, "imui-table-layout.row.1.cell.owner");

    let assert_close = |label: &str, a: f32, b: f32| {
        assert!((a - b).abs() <= 0.5, "{label} drifted: left={a}, right={b}");
    };

    assert_close(
        "status x header vs row0",
        header_status.origin.x.0,
        row0_status.origin.x.0,
    );
    assert_close(
        "status x header vs row1",
        header_status.origin.x.0,
        row1_status.origin.x.0,
    );
    assert_close(
        "status width header vs row0",
        header_status.size.width.0,
        row0_status.size.width.0,
    );
    assert_close(
        "status width header vs row1",
        header_status.size.width.0,
        row1_status.size.width.0,
    );

    assert_close(
        "owner x header vs row0",
        header_owner.origin.x.0,
        row0_owner.origin.x.0,
    );
    assert_close(
        "owner x header vs row1",
        header_owner.origin.x.0,
        row1_owner.origin.x.0,
    );
    assert_close(
        "owner width header vs row0",
        header_owner.size.width.0,
        row0_owner.size.width.0,
    );
    assert_close(
        "owner width header vs row1",
        header_owner.size.width.0,
        row1_owner.size.width.0,
    );
}

#[test]
fn virtual_list_helper_mounts_small_render_window_and_scrolls_to_target_row() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let scroll = VirtualListScrollHandle::new();
    let rendered_range = Rc::new(Cell::new(None::<(usize, usize)>));

    let rendered_out = rendered_range.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-virtual-list",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.virtual_list_with_options(
                    "imui-virtual-list",
                    100,
                    VirtualListOptions {
                        viewport_height: Px(60.0),
                        estimate_row_height: Px(20.0),
                        overscan: 0,
                        measure_mode: VirtualListMeasureMode::Fixed,
                        handle: Some(scroll.clone()),
                        test_id: Some(Arc::from("imui-virtual-list")),
                        ..Default::default()
                    },
                    |index| index as fret_ui::ItemKey,
                    |ui, index| {
                        let _ = ui.selectable(format!("Row {index}"), false);
                    },
                );
                rendered_out.set(response.rendered_range());
            })
        },
    );

    app.advance_frame();

    let rendered_out = rendered_range.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-virtual-list",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.virtual_list_with_options(
                    "imui-virtual-list",
                    100,
                    VirtualListOptions {
                        viewport_height: Px(60.0),
                        estimate_row_height: Px(20.0),
                        overscan: 0,
                        measure_mode: VirtualListMeasureMode::Fixed,
                        handle: Some(scroll.clone()),
                        test_id: Some(Arc::from("imui-virtual-list")),
                        ..Default::default()
                    },
                    |index| index as fret_ui::ItemKey,
                    |ui, index| {
                        let _ = ui.selectable(format!("Row {index}"), false);
                    },
                );
                rendered_out.set(response.rendered_range());
            })
        },
    );

    let range0 = rendered_range.get().expect("initial rendered range");
    assert_eq!(range0.0, 0);
    assert!(
        range0.1 <= 3,
        "initial rendered range too large: {range0:?}"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-virtual-list.row.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-virtual-list.row.50",
    ));

    scroll.scroll_to_item(50, fret_ui::ScrollStrategy::Start);
    app.advance_frame();

    let rendered_out = rendered_range.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-virtual-list",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.virtual_list_with_options(
                    "imui-virtual-list",
                    100,
                    VirtualListOptions {
                        viewport_height: Px(60.0),
                        estimate_row_height: Px(20.0),
                        overscan: 0,
                        measure_mode: VirtualListMeasureMode::Fixed,
                        handle: Some(scroll.clone()),
                        test_id: Some(Arc::from("imui-virtual-list")),
                        ..Default::default()
                    },
                    |index| index as fret_ui::ItemKey,
                    |ui, index| {
                        let _ = ui.selectable(format!("Row {index}"), index == 50);
                    },
                );
                rendered_out.set(response.rendered_range());
            })
        },
    );

    let range1 = rendered_range.get().expect("scrolled rendered range");
    assert!(
        range1.0 <= 50 && 50 <= range1.1,
        "target row not in range: {range1:?}"
    );
    assert!(range1.1.saturating_sub(range1.0) <= 3);
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-virtual-list.row.50",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-virtual-list.row.0",
    ));
}

#[test]
fn separator_text_helper_renders_label_with_trailing_rule() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
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
        "imui-separator-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Above",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-separator-text.above")),
                        ..Default::default()
                    },
                );
                ui.separator_text_with_options(
                    "Section",
                    fret_ui_kit::imui::SeparatorTextOptions {
                        test_id: Some(Arc::from("imui-separator-text.section")),
                    },
                );
                ui.menu_item_with_options(
                    "Below",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-separator-text.below")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let section = bounds_for_test_id(&ui, "imui-separator-text.section");
    let label = bounds_for_test_id(&ui, "imui-separator-text.section.label");
    let line = bounds_for_test_id(&ui, "imui-separator-text.section.line");

    assert!(section.size.width.0 > 200.0);
    assert!(label.origin.x.0 >= section.origin.x.0);
    assert!(line.origin.x.0 >= label.origin.x.0 + label.size.width.0);
    assert!(line.size.width.0 > 40.0);
    assert!(line.origin.x.0 + line.size.width.0 <= section.origin.x.0 + section.size.width.0 + 1.0);
}

#[test]
fn bullet_text_helper_renders_indicator_before_wrapped_label() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(280.0), Px(180.0)),
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
        "imui-bullet-text",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.bullet_text_with_options(
                    "Bullet text keeps informational copy separate from pressable controls even when the line wraps.",
                    fret_ui_kit::imui::BulletTextOptions {
                        test_id: Some(Arc::from("imui-bullet-text.entry")),
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let entry = bounds_for_test_id(&ui, "imui-bullet-text.entry");
    let indicator = bounds_for_test_id(&ui, "imui-bullet-text.entry.indicator");
    let label = bounds_for_test_id(&ui, "imui-bullet-text.entry.label");

    assert!(entry.size.width.0 > 160.0);
    assert!(indicator.origin.x.0 >= entry.origin.x.0);
    assert!(indicator.origin.x.0 + indicator.size.width.0 <= label.origin.x.0);
    assert!(label.origin.y.0 <= indicator.origin.y.0 + Px(12.0).0);
    assert!(label.size.height.0 > indicator.size.height.0);
}
// Note: `for_each_keyed` is exercised indirectly by downstream ecosystem crates. The core
// smoke tests above focus on interaction correctness (`clicked` / `changed`).
