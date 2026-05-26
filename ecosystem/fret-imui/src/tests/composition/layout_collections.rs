use super::*;
use fret_authoring::UiWriter as _;

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
fn porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    fret_ui::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = fret_ui::theme::ThemeConfig {
            name: "IMUI porting sugar test".to_string(),
            ..fret_ui::theme::ThemeConfig::default()
        };
        cfg.metrics
            .insert("component.imui.item_spacing_x_px".to_string(), 17.0);
        cfg.metrics
            .insert("component.imui.item_spacing_y_px".to_string(), 9.0);
        cfg.metrics
            .insert("component.imui.indent_spacing_px".to_string(), 33.0);
        theme.apply_config_patch(&cfg);
    });
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-porting-sugar-layout",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.items_with_options(
                    ItemFlowOptions {
                        test_id: Some(Arc::from("imui-porting.items")),
                        ..Default::default()
                    },
                    |ui| {
                        ui.same_line_with_options(
                            SameLineOptions {
                                test_id: Some(Arc::from("imui-porting.same-line")),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.menu_item_with_options(
                                    "Alpha",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-porting.same-line.alpha")),
                                        ..Default::default()
                                    },
                                );
                                ui.dummy_with_options(
                                    Size::new(Px(12.0), Px(6.0)),
                                    fret_ui_kit::imui::DummyOptions {
                                        test_id: Some(Arc::from("imui-porting.same-line.dummy")),
                                    },
                                );
                                let _ = ui.menu_item_with_options(
                                    "Beta",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-porting.same-line.beta")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );
                        ui.spacing_with_options(SpacingOptions {
                            test_id: Some(Arc::from("imui-porting.spacing")),
                            ..Default::default()
                        });
                        ui.indent_with_options(
                            IndentOptions {
                                test_id: Some(Arc::from("imui-porting.indent")),
                                content_test_id: Some(Arc::from("imui-porting.indent.content")),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.menu_item_with_options(
                                    "Indented",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from("imui-porting.indent.row")),
                                        ..Default::default()
                                    },
                                );
                            },
                        );
                        ui.dummy_with_options(
                            Size::new(Px(30.0), Px(10.0)),
                            fret_ui_kit::imui::DummyOptions {
                                test_id: Some(Arc::from("imui-porting.dummy")),
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alpha = bounds_for_test_id(&ui, "imui-porting.same-line.alpha");
    let same_line_dummy = bounds_for_test_id(&ui, "imui-porting.same-line.dummy");
    let beta = bounds_for_test_id(&ui, "imui-porting.same-line.beta");
    let same_line_gap = same_line_dummy.origin.x.0 - (alpha.origin.x.0 + alpha.size.width.0);
    assert!(
        (same_line_gap - 17.0).abs() <= 0.5,
        "same_line should use the item_spacing_x token: gap={same_line_gap} alpha={alpha:?} dummy={same_line_dummy:?}"
    );
    let same_line_dummy_gap =
        beta.origin.x.0 - (same_line_dummy.origin.x.0 + same_line_dummy.size.width.0);
    assert!(
        (same_line_dummy.size.width.0 - 12.0).abs() <= 0.5
            && (same_line_dummy.size.height.0 - 6.0).abs() <= 0.5
            && (same_line_dummy_gap - 17.0).abs() <= 0.5,
        "dummy should preserve explicit size and participate in same_line gaps: gap={same_line_dummy_gap} dummy={same_line_dummy:?} beta={beta:?}"
    );

    let same_line = bounds_for_test_id(&ui, "imui-porting.same-line");
    let spacing = bounds_for_test_id(&ui, "imui-porting.spacing");
    let vertical_gap = spacing.origin.y.0 - (same_line.origin.y.0 + same_line.size.height.0);
    assert!(
        (vertical_gap - 9.0).abs() <= 0.5,
        "items should use the item_spacing_y token between rows: gap={vertical_gap} same_line={same_line:?} spacing={spacing:?}"
    );
    assert!(
        (spacing.size.height.0 - 9.0).abs() <= 0.5,
        "spacing() should default to one item_spacing_y row: spacing={spacing:?}"
    );

    let indent = bounds_for_test_id(&ui, "imui-porting.indent");
    let indent_row = bounds_for_test_id(&ui, "imui-porting.indent.row");
    let indent_offset = indent_row.origin.x.0 - indent.origin.x.0;
    assert!(
        (indent_offset - 33.0).abs() <= 0.5,
        "indent should use the indent_spacing token: offset={indent_offset} indent={indent:?} row={indent_row:?}"
    );

    let dummy = bounds_for_test_id(&ui, "imui-porting.dummy");
    assert!(
        (dummy.size.width.0 - 30.0).abs() <= 0.5 && (dummy.size.height.0 - 10.0).abs() <= 0.5,
        "dummy should preserve explicit size: dummy={dummy:?}"
    );
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
                    layout: fret_ui_kit::LayoutRefinement::default().h_px(Px(84.0)),
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

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let region = bounds_for_test_id(&ui, "imui-child-region");
    let viewport = bounds_for_test_id(&ui, "imui-child-region.viewport");
    let content = bounds_for_test_id(&ui, "imui-child-region.content");
    let row0 = bounds_for_test_id(&ui, "imui-child-region.row.0");
    let row1 = bounds_for_test_id(&ui, "imui-child-region.row.1");
    assert!(
        row1.origin.y.0 >= row0.origin.y.0 + row0.size.height.0,
        "child-region rows should stack before scrolling: region={region:?} viewport={viewport:?} content={content:?} row0={row0:?} row1={row1:?}"
    );
    assert!(
        handle.max_offset().y.0 > 0.0,
        "child-region should expose a real vertical scroll range before the offset is changed"
    );

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

    assert!(
        handle.offset().y.0 > 0.0,
        "child-region scroll handle should keep the requested vertical offset when content overflows"
    );
}

#[test]
fn list_box_container_stamps_semantics_scroll_and_hosts_selectables() {
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
            ui.list_box_with_options(
                "asset-list-box",
                ListBoxOptions {
                    layout: fret_ui_kit::LayoutRefinement::default().h_px(Px(72.0)),
                    scroll: ScrollOptions {
                        handle: Some(handle.clone()),
                        viewport_test_id: Some(Arc::from("imui-list-box.viewport")),
                        ..Default::default()
                    },
                    label: Some(Arc::from("Assets")),
                    multiselectable: true,
                    test_id: Some(Arc::from("imui-list-box")),
                    content_test_id: Some(Arc::from("imui-list-box.content")),
                },
                |ui| {
                    for index in 0..12 {
                        let _ = ui.selectable_with_options(
                            format!("Asset {index}"),
                            SelectableOptions {
                                selected: index == 2,
                                test_id: Some(Arc::from(format!("imui-list-box.row.{index}"))),
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
        "imui-list-box",
        render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-list-box",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-list-box.viewport",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-list-box.content",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let listbox = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-list-box"))
        .expect("listbox semantics node");
    assert_eq!(listbox.role, SemanticsRole::ListBox);
    assert_eq!(listbox.label.as_deref(), Some("Assets"));
    assert!(listbox.flags.multiselectable);
    assert!(
        listbox.active_descendant.is_none(),
        "list_box container must not own active-descendant policy"
    );

    let row2 = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-list-box.row.2"))
        .expect("selected row semantics node");
    assert_eq!(row2.role, SemanticsRole::ListBoxOption);
    assert!(row2.flags.selected);

    let row0 = bounds_for_test_id(&ui, "imui-list-box.row.0");
    let row1 = bounds_for_test_id(&ui, "imui-list-box.row.1");
    assert!(
        row1.origin.y.0 >= row0.origin.y.0 + row0.size.height.0,
        "listbox rows should stack vertically: row0={row0:?} row1={row1:?}"
    );
    assert!(
        handle.max_offset().y.0 > 0.0,
        "listbox should expose a real vertical scroll range"
    );
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
fn child_region_helper_renders_resize_y_handle_without_breaking_scroll_chrome() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(300.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.child_region_with_options(
                "imui-child-region.resize-y",
                ChildRegionOptions {
                    layout: fret_ui_kit::LayoutRefinement::default()
                        .w_px(Px(220.0))
                        .h_px(Px(96.0)),
                    resize_y: Some(
                        ChildRegionResizeYOptions::new()
                            .min_height(Px(48.0))
                            .max_height(Px(160.0))
                            .handle_test_id("imui-child-region.resize-y.handle"),
                    ),
                    scroll: fret_ui_kit::imui::ScrollOptions {
                        viewport_test_id: Some(Arc::from("imui-child-region.resize-y.viewport")),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-child-region.resize-y")),
                    content_test_id: Some(Arc::from("imui-child-region.resize-y.content")),
                    ..Default::default()
                },
                |ui| {
                    ui.menu_item_with_options(
                        "Resizable row",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-child-region.resize-y.row")),
                            ..Default::default()
                        },
                    );
                },
            );

            assert!(response.resize_y().enabled());
            assert_eq!(response.resize_y().min_height(), Some(Px(48.0)));
            assert_eq!(response.resize_y().max_height(), Some(Px(160.0)));
            assert_eq!(response.resize_y().height_from_start(Px(96.0)), Px(96.0));
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region-resize-y",
        render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.resize-y",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.resize-y.viewport",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.resize-y.content",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.resize-y.handle",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region.resize-y.row",
    ));

    let region = bounds_for_test_id(&ui, "imui-child-region.resize-y");
    let handle = bounds_for_test_id(&ui, "imui-child-region.resize-y.handle");
    assert!(handle.origin.y.0 >= region.origin.y.0 + region.size.height.0 - 7.0);
    assert!(handle.size.width.0 >= region.size.width.0 - 1.0);
}

#[test]
fn child_region_without_height_constraint_auto_sizes_to_content() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region-auto-height",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.vertical_with_options(
                    VerticalOptions {
                        gap: Px(8.0).into(),
                        ..Default::default()
                    },
                    |ui| {
                        ui.child_region_with_options(
                            "imui-child-region.auto-height",
                            ChildRegionOptions {
                                layout: fret_ui_kit::LayoutRefinement::default().w_px(Px(180.0)),
                                test_id: Some(Arc::from("imui-child-region.auto-height")),
                                content_test_id: Some(Arc::from(
                                    "imui-child-region.auto-height.content",
                                )),
                                scroll: fret_ui_kit::imui::ScrollOptions {
                                    viewport_test_id: Some(Arc::from(
                                        "imui-child-region.auto-height.viewport",
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |ui| {
                                for index in 0..3 {
                                    ui.menu_item_with_options(
                                        format!("Auto row {index}"),
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(format!(
                                                "imui-child-region.auto-height.row.{index}",
                                            ))),
                                            ..Default::default()
                                        },
                                    );
                                }
                            },
                        );
                        ui.menu_item_with_options(
                            "After",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-child-region.auto-height.after")),
                                ..Default::default()
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = bounds_for_test_id(&ui, "imui-child-region.auto-height");
    let viewport = bounds_for_test_id(&ui, "imui-child-region.auto-height.viewport");
    let content = bounds_for_test_id(&ui, "imui-child-region.auto-height.content");
    let row0 = bounds_for_test_id(&ui, "imui-child-region.auto-height.row.0");
    let row2 = bounds_for_test_id(&ui, "imui-child-region.auto-height.row.2");
    let after = bounds_for_test_id(&ui, "imui-child-region.auto-height.after");

    assert_eq!(region.size.width, Px(180.0));
    assert!(
        region.size.height.0 >= content.size.height.0,
        "auto-height child region should contain measured content: region={region:?} content={content:?}"
    );
    assert!(
        viewport.size.height.0 >= content.size.height.0,
        "unbounded child-region viewport should remain auto-height instead of forcing a scroll box"
    );
    assert!(
        row2.origin.y.0 >= row0.origin.y.0 + row0.size.height.0,
        "rows should stack inside the auto-height child region: region={region:?} viewport={viewport:?} content={content:?} row0={row0:?} row2={row2:?}"
    );
    assert!(
        after.origin.y.0 >= region.origin.y.0 + region.size.height.0,
        "following siblings should be pushed below the auto-height child region"
    );
}

#[test]
fn child_region_without_width_constraint_auto_sizes_to_content() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region-auto-width",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal_with_options(
                    HorizontalOptions {
                        gap: Px(8.0).into(),
                        items: fret_ui_kit::Items::Start,
                        ..Default::default()
                    },
                    |ui| {
                        ui.child_region_with_options(
                            "imui-child-region.auto-width",
                            ChildRegionOptions {
                                layout: fret_ui_kit::LayoutRefinement::default().h_px(Px(96.0)),
                                test_id: Some(Arc::from("imui-child-region.auto-width")),
                                content_test_id: Some(Arc::from(
                                    "imui-child-region.auto-width.content",
                                )),
                                scroll: fret_ui_kit::imui::ScrollOptions {
                                    viewport_test_id: Some(Arc::from(
                                        "imui-child-region.auto-width.viewport",
                                    )),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            |ui| {
                                ui.menu_item_with_options(
                                    "Wide row",
                                    MenuItemOptions {
                                        test_id: Some(Arc::from(
                                            "imui-child-region.auto-width.row",
                                        )),
                                        ..Default::default()
                                    },
                                );
                            },
                        );
                        ui.menu_item_with_options(
                            "After",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-child-region.auto-width.after")),
                                ..Default::default()
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let region = bounds_for_test_id(&ui, "imui-child-region.auto-width");
    let viewport = bounds_for_test_id(&ui, "imui-child-region.auto-width.viewport");
    let content = bounds_for_test_id(&ui, "imui-child-region.auto-width.content");
    let row = bounds_for_test_id(&ui, "imui-child-region.auto-width.row");
    let after = bounds_for_test_id(&ui, "imui-child-region.auto-width.after");

    assert_eq!(region.size.height, Px(96.0));
    assert!(
        region.size.width.0 >= content.size.width.0,
        "auto-width child region should contain measured content: region={region:?} content={content:?}"
    );
    assert!(
        viewport.size.width.0 >= content.size.width.0,
        "unbounded child-region viewport should remain auto-width instead of forcing a scroll box"
    );
    assert!(
        content.size.width.0 >= row.size.width.0,
        "content should include the measured row width: content={content:?} row={row:?}"
    );
    assert!(
        after.origin.x.0 >= region.origin.x.0 + region.size.width.0,
        "following siblings should be pushed after the auto-width child region"
    );
    assert!(
        region.size.width.0 < bounds.size.width.0 - 80.0,
        "auto-width child region should not fill the entire available row: region={region:?}"
    );
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
fn table_helper_skips_hidden_columns_in_header_and_body() {
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

    let columns = [
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)).hidden(),
        TableColumn::px("Owner", Px(88.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-hidden-column",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-hidden-column",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-hidden-column")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.name"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.owner"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.status"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.row.0.cell.status"
    ));
}

#[test]
fn table_helper_pins_left_and_right_columns_while_center_columns_scroll() {
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
    let scroll = fret_ui::scroll::ScrollHandle::default();

    let columns = [
        TableColumn::px("ID###id", Px(48.0)).pinned_left(),
        TableColumn::px("Name###name", Px(180.0)),
        TableColumn::px("Kind###kind", Px(160.0)),
        TableColumn::px("Score###score", Px(64.0)).pinned_right(),
    ];

    let build = |cx: &mut ElementContext<'_, TestHost>| {
        let scroll = scroll.clone();
        crate::imui_raw(cx, |ui| {
            ui.table_with_options(
                "imui-table-pinned-columns",
                &columns,
                TableOptions {
                    horizontal_scroll: Some(scroll),
                    test_id: Some(Arc::from("imui-table-pinned-columns")),
                    ..Default::default()
                },
                |table| {
                    table.row("alpha", |row| {
                        row.cell_text("01");
                        row.cell_text("Alpha asset");
                        row.cell_text("Texture");
                        row.cell_text("98");
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
        "imui-table-pinned-columns",
        build,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before_id = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.id");
    let before_name = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.name");
    let before_score = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.score");

    assert_eq!(columns[0].pin(), TableColumnPin::Left);
    assert_eq!(columns[3].pin(), TableColumnPin::Right);
    assert!(
        scroll.max_offset().x.0 > 0.0,
        "expected center columns to create a horizontal scroll range"
    );

    scroll.set_offset(Point::new(Px(96.0), Px(0.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-pinned-columns",
        build,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_id = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.id");
    let after_name = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.name");
    let after_score = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.score");

    assert!(
        (after_id.origin.x.0 - before_id.origin.x.0).abs() <= 0.5,
        "left pinned column should not move with center scroll: before={before_id:?} after={after_id:?}"
    );
    assert!(
        (after_score.origin.x.0 - before_score.origin.x.0).abs() <= 0.5,
        "right pinned column should not move with center scroll: before={before_score:?} after={after_score:?}"
    );
    assert!(
        after_name.origin.x.0 < before_name.origin.x.0 - 8.0,
        "center column should move left with horizontal scroll: before={before_name:?} after={after_name:?}"
    );
}

#[test]
fn table_helper_applies_runtime_column_visibility_state() {
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

    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)).hidden(),
    ];
    let visibility = ImUiTableColumnVisibilityState::new([
        (Arc::from("status"), false),
        (Arc::from("owner"), true),
    ]);
    let columns = visibility.apply_to_columns(&columns);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-runtime-column-visibility",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-runtime-column-visibility",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-runtime-column-visibility")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.name"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.status"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.owner"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.name"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.status"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.owner"
    ));
}

#[test]
fn table_column_visibility_menu_item_updates_visibility_state() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let column = TableColumn::px("Status###status", Px(96.0));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = table_column_visibility_menu_item(
                    ui,
                    &column,
                    &model,
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu.status")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu.status",
    );
    click_at(&mut ui, &mut app, &mut services, item);

    app.advance_frame();
    let changed = Rc::new(Cell::new(false));
    let visible = Rc::new(Cell::new(true));
    let changed_out = changed.clone();
    let visible_out = visible.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_item(
                    ui,
                    &column,
                    &model,
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu.status")),
                        ..Default::default()
                    },
                )
                .expect("column has stable id");
                changed_out.set(response.changed());
                let value = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .map(|state| state.is_visible("status", column.visible()))
                    .unwrap_or(column.visible());
                visible_out.set(value);
            })
        },
    );

    assert!(changed.get());
    assert!(!visible.get());
}

#[test]
fn table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns() {
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

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))).with_id("actions"),
        TableColumn::px("###internal", Px(48.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_items(
                    ui,
                    &columns,
                    &model,
                    TableColumnVisibilityMenuOptions {
                        test_id_prefix: Some(Arc::from(
                            "imui-table-column-visibility-menu-items.item.",
                        )),
                        ..Default::default()
                    },
                );
                assert_eq!(response.len(), 2);
                assert!(response.item("name").is_some());
                assert!(response.item("status").is_some());
                assert!(response.item("actions").is_none());
                assert!(response.item("internal").is_none());
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.name",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.status",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.actions",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.internal",
    ));

    let status = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.status",
    );
    click_at(&mut ui, &mut app, &mut services, status);

    app.advance_frame();
    let changed = Rc::new(Cell::new(false));
    let visible = Rc::new(Cell::new(true));
    let changed_out = changed.clone();
    let visible_out = visible.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_items(
                    ui,
                    &columns,
                    &model,
                    TableColumnVisibilityMenuOptions {
                        test_id_prefix: Some(Arc::from(
                            "imui-table-column-visibility-menu-items.item.",
                        )),
                        ..Default::default()
                    },
                );
                changed_out.set(response.changed());
                visible_out.set(
                    response
                        .item("status")
                        .expect("status item response")
                        .visible(),
                );
            })
        },
    );

    assert!(changed.get());
    assert!(!visible.get());

    let applied_visible = app
        .models()
        .get_cloned(&model)
        .expect("visibility model")
        .apply_to_columns(&columns);
    assert!(applied_visible[0].visible());
    assert!(!applied_visible[1].visible());
    assert!(applied_visible[2].visible());
    assert!(applied_visible[3].visible());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                ui.table_with_options(
                    "imui-table-column-visibility-menu-items-applied",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu-items-applied")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Open");
                            row.cell_text("Internal");
                        });
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
        "imui-table-column-visibility-menu-items-applied.header.cell.name",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.header.cell.status",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.row.0.cell.name",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.row.0.cell.status",
    ));
}

#[test]
fn table_column_visibility_header_context_menu_opens_and_updates_state() {
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

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];

    let opened = Rc::new(Cell::new(false));
    let render = {
        let model = model.clone();
        let opened = opened.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                let response = ui.table_with_options(
                    "imui-table-header-visibility-menu",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-visibility-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
                let menu = table_column_visibility_header_context_menu(
                    ui,
                    "imui-table-header-visibility-menu.columns",
                    &response,
                    &columns,
                    &model,
                    TableColumnVisibilityHeaderContextMenuOptions {
                        menu: TableColumnVisibilityMenuOptions {
                            test_id_prefix: Some(Arc::from(
                                "imui-table-header-visibility-menu.menu.item.",
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );
                opened.set(menu.open());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(!opened.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    ));

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(opened.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.name",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    ));

    let status_item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    );
    click_at(&mut ui, &mut app, &mut services, status_item);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(
        !app.models()
            .get_cloned(&model)
            .expect("visibility model")
            .is_visible("status", true)
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.header.cell.status",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.row.0.cell.status",
    ));
}

#[test]
fn table_column_visibility_header_context_menu_opens_from_plain_header() {
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

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];

    let opened = Rc::new(Cell::new(false));
    let plain_header_clicked = Rc::new(Cell::new(true));
    let render = {
        let model = model.clone();
        let opened = opened.clone();
        let plain_header_clicked = plain_header_clicked.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                let response = ui.table_with_options(
                    "imui-table-plain-header-visibility-menu",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-visibility-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
                plain_header_clicked.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .clicked(),
                );
                let menu = table_column_visibility_header_context_menu(
                    ui,
                    "imui-table-plain-header-visibility-menu.columns",
                    &response,
                    &columns,
                    &model,
                    TableColumnVisibilityHeaderContextMenuOptions {
                        menu: TableColumnVisibilityMenuOptions {
                            test_id_prefix: Some(Arc::from(
                                "imui-table-plain-header-visibility-menu.menu.item.",
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );
                opened.set(menu.open());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-visibility-menu",
        &render,
    );
    assert!(!opened.get());
    assert!(!plain_header_clicked.get());

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-visibility-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-visibility-menu",
        &render,
    );
    assert!(opened.get());
    assert!(!plain_header_clicked.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-visibility-menu.menu.item.status",
    ));
}

#[test]
fn table_plain_header_left_click_does_not_activate_or_click() {
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

    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let clicked = Rc::new(Cell::new(true));
    let activated = Rc::new(Cell::new(true));
    let deactivated = Rc::new(Cell::new(true));
    let render = {
        let clicked = clicked.clone();
        let activated = activated.clone();
        let deactivated = deactivated.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-plain-header-left-click",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-left-click")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                clicked.set(header.clicked());
                activated.set(header.activated());
                deactivated.set(header.deactivated());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-left-click",
        &render,
    );
    assert!(!clicked.get());
    assert!(!activated.get());
    assert!(!deactivated.get());

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-left-click.header.cell.name",
    );
    click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-left-click",
        &render,
    );
    assert!(!clicked.get());
    assert!(!activated.get());
    assert!(!deactivated.get());
}

#[test]
fn table_plain_header_reports_context_menu_request_from_keyboard_without_clicking() {
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

    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let clicked = Rc::new(Cell::new(false));
    let header_id = Rc::new(Cell::new(None));
    let header_id_out = header_id.clone();

    let render = {
        let requested = requested.clone();
        let clicked = clicked.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-plain-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                header_id_out.set(header.id());
                requested.set(header.context_menu_requested());
                clicked.set(header.clicked());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(!requested.get());
    assert!(!clicked.get());

    let header_id = header_id.get().expect("plain header response id");
    ui.request_focus_element(&mut app, header_id);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.focus().is_some(),
        "expected plain header trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(requested.get());
    assert!(!clicked.get());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(!requested.get());
    assert!(!clicked.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(requested.get());
    assert!(!clicked.get());
}

#[test]
fn table_sortable_header_reports_context_menu_request_on_right_click() {
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

    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let anchor_matches_click = Rc::new(Cell::new(false));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                assert!(
                    !response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested()
                );
            })
        },
    );

    let at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-context-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let anchor_matches_click_out = anchor_matches_click.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                requested_out.set(header.context_menu_requested());
                anchor_matches_click_out.set(header.context_menu_anchor() == Some(at));
            })
        },
    );

    assert!(requested.get());
    assert!(anchor_matches_click.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(!requested.get());
}

#[test]
fn table_sortable_header_reports_context_menu_request_from_keyboard() {
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

    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let header_id = Rc::new(Cell::new(None));
    let header_id_out = header_id.clone();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                header_id_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .id(),
                );
                assert!(
                    !response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested()
                );
            })
        },
    );

    let header_id = header_id.get().expect("name header response id");
    ui.request_focus_element(&mut app, header_id);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.focus().is_some(),
        "expected sortable header trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(requested.get());
}

#[test]
fn table_helper_applies_explicit_row_and_cell_background_overrides() {
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

    let row_bg = fret_core::Color::from_srgb_hex_rgb(0x201010);
    let cell_bg = fret_core::Color::from_srgb_hex_rgb(0x102020);
    let columns = [
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-background-overrides",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-background-overrides",
                    &columns,
                    TableOptions {
                        striped: true,
                        test_id: Some(Arc::from("imui-table-background-overrides")),
                        ..Default::default()
                    },
                    |table| {
                        table.row_with_options(
                            "alpha",
                            fret_ui_kit::imui::TableRowOptions {
                                test_id: Some(Arc::from("imui-table-background-overrides.row")),
                                background: Some(row_bg),
                            },
                            |row| {
                                row.cell_text("Alpha");
                                row.cell_text_with_options(
                                    "Ready",
                                    fret_ui_kit::imui::TableCellOptions {
                                        test_id: Some(Arc::from(
                                            "imui-table-background-overrides.cell",
                                        )),
                                        background: Some(cell_bg),
                                    },
                                );
                            },
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-background-overrides.row"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-background-overrides.cell"
    ));

    services.prepared.clear();
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let ops = scene.ops();

    let row_bg_index = first_solid_quad_index(ops, row_bg).expect("row background quad");
    let cell_bg_index = first_solid_quad_index(ops, cell_bg).expect("cell background quad");
    assert!(
        row_bg_index < cell_bg_index,
        "expected cell background to paint after row background, got scene ops: {ops:?}"
    );
}

fn first_solid_quad_index(ops: &[fret_core::SceneOp], color: fret_core::Color) -> Option<usize> {
    ops.iter().position(|op| {
        matches!(
            op,
            fret_core::SceneOp::Quad {
                background:
                    fret_core::scene::PaintBindingV1 {
                        paint: fret_core::scene::Paint::Solid(actual),
                        ..
                    },
                ..
            } if *actual == color
        )
    })
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
fn virtual_list_fixed_rows_clip_oversized_row_content() {
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
            ui.virtual_list_with_options(
                "imui-virtual-list-fixed-clip",
                4,
                VirtualListOptions {
                    viewport_height: Px(72.0),
                    estimate_row_height: Px(20.0),
                    overscan: 0,
                    measure_mode: VirtualListMeasureMode::Fixed,
                    test_id: Some(Arc::from("imui-virtual-list-fixed-clip")),
                    ..Default::default()
                },
                |index| index as fret_ui::ItemKey,
                |ui, index| {
                    let content = ui.with_cx_mut(|cx| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.height = Length::Px(Px(64.0));
                        cx.container(props, |_cx| Vec::new())
                            .test_id(Arc::from(format!(
                                "imui-virtual-list-fixed-clip.content.{index}"
                            )))
                    });
                    ui.add(content);
                },
            );
        })
    };

    for _ in 0..3 {
        run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-virtual-list-fixed-clip",
            &render,
        );
        app.advance_frame();
    }

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let row_semantics = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-virtual-list-fixed-clip.row.0",
    );
    let row_node = ui
        .debug_node_children(row_semantics)
        .into_iter()
        .next()
        .expect("row semantics should wrap the fixed-row container");
    let row = ui.debug_node_bounds(row_node).expect("row bounds");
    let content = bounds_for_test_id(&ui, "imui-virtual-list-fixed-clip.content.0");

    assert!(
        row.size.height.0 <= 20.5,
        "fixed virtual-list row should keep the configured row height, got {row:?}"
    );
    assert_eq!(
        ui.debug_node_clips_hit_test(row_node),
        Some(true),
        "fixed virtual-list row should clip oversized row contents"
    );
    assert!(
        content.size.height.0 > row.size.height.0,
        "test must exercise oversized row content, row={row:?}, content={content:?}"
    );
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
