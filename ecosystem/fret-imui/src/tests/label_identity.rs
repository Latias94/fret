use super::*;

use fret_ui_kit::imui::{
    BeginMenuOptions, ButtonOptions, CheckboxOptions, CollapsingHeaderOptions, ComboOptions,
    MenuBarOptions, MenuItemOptions, RadioOptions, SelectableOptions, SeparatorTextOptions,
    SliderOptions, SwitchOptions, TabBarOptions, TabItemOptions, TableColumn, TableOptions,
    TreeNodeOptions,
};

fn current_focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
) -> Option<String> {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let focus = ui.focus()?;
    let snap = ui.semantics_snapshot()?;
    snap.nodes
        .iter()
        .find(|node| node.id == focus)
        .and_then(|node| node.test_id.as_deref().map(str::to_owned))
}

#[test]
fn label_identity_button_suffixes_hide_from_text_and_preserve_focus_across_reorder() {
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

    let flipped = Rc::new(Cell::new(false));
    let progress = Rc::new(Cell::new(41));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let task_a = (
                    format!("Task {}%###task-a", progress.get()),
                    "imui-label-identity.task-a",
                );
                let task_b = (
                    String::from("Task 99%###task-b"),
                    "imui-label-identity.task-b",
                );
                let items = if flipped.get() {
                    vec![task_b, task_a]
                } else {
                    vec![task_a, task_b]
                };

                for (label, test_id) in items {
                    let _ = ui.button_with_options(
                        label,
                        ButtonOptions {
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                    );
                }

                let _ = ui.button_with_options(
                    "Play##toolbar",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-label-identity.play")),
                        ..Default::default()
                    },
                );
                let _ = ui.button_with_options(
                    "##hidden-button",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-label-identity.hidden")),
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
        "imui-label-identity",
        |cx| render(cx),
    );

    let _task_a = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.task-a",
    );

    progress.set(42);
    flipped.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity",
        &render,
    );

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-label-identity.task-a"))
    );
    assert!(services.prepared.iter().any(|text| text == "Task 41%"));
    assert!(services.prepared.iter().any(|text| text == "Task 42%"));
    assert!(services.prepared.iter().any(|text| text == "Play"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("toolbar")
            || text.contains("hidden-button")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}

#[test]
fn label_identity_selectable_and_menu_item_suffixes_hide_from_text() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let flipped = Rc::new(Cell::new(false));
    let progress = Rc::new(Cell::new(10));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let row_a = (
                    format!("Row {}###row-a", progress.get()),
                    "imui-label-identity.selectable.a",
                );
                let row_b = (
                    String::from("Row B###row-b"),
                    "imui-label-identity.selectable.b",
                );
                let rows = if flipped.get() {
                    vec![row_b, row_a]
                } else {
                    vec![row_a, row_b]
                };
                for (label, test_id) in rows {
                    let _ = ui.selectable_with_options(
                        label,
                        SelectableOptions {
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                    );
                }

                ui.menu_bar_with_options(
                    MenuBarOptions {
                        test_id: Some(Arc::from("imui-label-identity.menu.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "labels",
                            "Labels",
                            BeginMenuOptions {
                                test_id: Some(Arc::from("imui-label-identity.menu.labels")),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.menu_item_with_options(
                                    "Open##primary",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-label-identity.menu.open")),
                                        ..Default::default()
                                    },
                                );
                                let _ = ui.menu_item_with_options(
                                    "Save 10###stable-save",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-label-identity.menu.save")),
                                        ..Default::default()
                                    },
                                );
                                let _ = ui.menu_item_with_options(
                                    "##hidden-menu-row",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-label-identity.menu.hidden")),
                                        ..Default::default()
                                    },
                                );
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
        "imui-label-identity-menu-selectable",
        |cx| render(cx),
    );

    let _row_a = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.selectable.a",
    );

    progress.set(11);
    flipped.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-menu-selectable",
        &render,
    );

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-label-identity.selectable.a"))
    );

    let menu_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.menu.labels",
    );
    click_at(&mut ui, &mut app, &mut services, menu_trigger);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-menu-selectable",
        &render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.menu.open"
    ));
    assert!(services.prepared.iter().any(|text| text == "Row 10"));
    assert!(services.prepared.iter().any(|text| text == "Row 11"));
    assert!(services.prepared.iter().any(|text| text == "Open"));
    assert!(services.prepared.iter().any(|text| text == "Save 10"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("primary")
            || text.contains("stable-save")
            || text.contains("hidden-menu-row")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}

#[test]
fn label_identity_model_controls_hide_suffixes_and_preserve_focus_across_reorder() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(300.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let flipped = Rc::new(Cell::new(false));
    let progress = Rc::new(Cell::new(10));
    let checkbox_a = app.models_mut().insert(false);
    let checkbox_b = app.models_mut().insert(false);
    let switch = app.models_mut().insert(false);
    let slider = app.models_mut().insert(0.25f32);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let row_a = (
                    format!("Check {}###check-a", progress.get()),
                    checkbox_a.clone(),
                    "imui-label-identity.checkbox.a",
                );
                let row_b = (
                    String::from("Check B###check-b"),
                    checkbox_b.clone(),
                    "imui-label-identity.checkbox.b",
                );
                let rows = if flipped.get() {
                    vec![row_b, row_a]
                } else {
                    vec![row_a, row_b]
                };
                for (label, model, test_id) in rows {
                    let _ = ui.checkbox_model_with_options(
                        label,
                        &model,
                        CheckboxOptions {
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                    );
                }

                let _ = ui.radio_with_options(
                    "Radio###radio-stable",
                    false,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-label-identity.radio")),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Switch##switch-id",
                    &switch,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-label-identity.switch")),
                        ..Default::default()
                    },
                );
                let _ = ui.slider_f32_model_with_options(
                    "Amount##slider-id",
                    &slider,
                    SliderOptions {
                        test_id: Some(Arc::from("imui-label-identity.slider")),
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
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
        "imui-label-identity-model-controls",
        |cx| render(cx),
    );

    let _checkbox_a = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.checkbox.a",
    );

    progress.set(11);
    flipped.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-model-controls",
        &render,
    );

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-label-identity.checkbox.a"))
    );
    assert!(services.prepared.iter().any(|text| text == "Check 10"));
    assert!(services.prepared.iter().any(|text| text == "Check 11"));
    assert!(services.prepared.iter().any(|text| text == "Radio"));
    assert!(services.prepared.iter().any(|text| text == "Switch"));
    assert!(services.prepared.iter().any(|text| text == "Amount"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("check-a")
            || text.contains("check-b")
            || text.contains("radio-stable")
            || text.contains("switch-id")
            || text.contains("slider-id")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}

#[test]
fn label_identity_explicit_id_controls_hide_suffixes_from_visible_labels() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(260.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_with_options(
                    "identity-combo",
                    "Mode##combo-label",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-label-identity.combo")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-label-identity.combo.alpha")),
                                ..Default::default()
                            },
                        );
                    },
                );

                ui.menu_bar_with_options(
                    MenuBarOptions {
                        test_id: Some(Arc::from("imui-label-identity.explicit-menu.root")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.begin_menu_with_options(
                            "file",
                            "File###file-menu-label",
                            BeginMenuOptions {
                                test_id: Some(Arc::from("imui-label-identity.explicit-menu.file")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );

                ui.tab_bar_with_options(
                    "identity-tabs",
                    TabBarOptions {
                        test_id: Some(Arc::from("imui-label-identity.tabs.root")),
                        ..Default::default()
                    },
                    |tabs| {
                        tabs.begin_tab_item_with_options(
                            "scene",
                            "Scene##scene-tab-label",
                            TabItemOptions {
                                default_selected: true,
                                test_id: Some(Arc::from("imui-label-identity.tabs.scene")),
                                panel_test_id: Some(Arc::from(
                                    "imui-label-identity.tabs.scene.panel",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                ui.text("Scene Panel");
                            },
                        );
                    },
                );

                let _ = ui.collapsing_header_with_options(
                    "identity-header",
                    "Header###header-label",
                    CollapsingHeaderOptions {
                        test_id: Some(Arc::from("imui-label-identity.header.root")),
                        header_test_id: Some(Arc::from("imui-label-identity.header")),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Header Panel");
                    },
                );
                let _ = ui.tree_node_with_options(
                    "identity-tree",
                    "Tree##tree-label",
                    TreeNodeOptions {
                        test_id: Some(Arc::from("imui-label-identity.tree")),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Tree Panel");
                    },
                );
                ui.separator_text_with_options(
                    "Section##section-label",
                    SeparatorTextOptions {
                        test_id: Some(Arc::from("imui-label-identity.separator")),
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
        "imui-label-identity-explicit-id-controls",
        |cx| render(cx),
    );

    assert!(services.prepared.iter().any(|text| text == "Mode"));
    assert!(services.prepared.iter().any(|text| text == "File"));
    assert!(services.prepared.iter().any(|text| text == "Scene"));
    assert!(services.prepared.iter().any(|text| text == "Header"));
    assert!(services.prepared.iter().any(|text| text == "Tree"));
    assert!(services.prepared.iter().any(|text| text == "Section"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("combo-label")
            || text.contains("file-menu-label")
            || text.contains("scene-tab-label")
            || text.contains("header-label")
            || text.contains("tree-label")
            || text.contains("section-label")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}

#[test]
fn label_identity_table_headers_hide_suffixes_from_visible_labels() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let columns = [
                TableColumn::fill("Name##asset-name-column"),
                TableColumn::px("Status###status-column", Px(120.0)),
            ];
            ui.table_with_options(
                "identity-table",
                &columns,
                TableOptions {
                    test_id: Some(Arc::from("imui-label-identity.table")),
                    ..Default::default()
                },
                |table| {
                    table.row("asset-a", |row| {
                        row.cell_text("Asset A");
                        row.cell_text("Ready");
                    });
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
        "imui-label-identity-table-headers",
        |cx| render(cx),
    );

    assert!(services.prepared.iter().any(|text| text == "Name"));
    assert!(services.prepared.iter().any(|text| text == "Status"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("asset-name-column")
            || text.contains("status-column")),
        "table header label suffixes should not be painted: {:?}",
        services.prepared
    );
}
