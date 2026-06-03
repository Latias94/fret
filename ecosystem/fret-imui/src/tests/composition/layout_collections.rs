use super::*;
use fret_authoring::UiWriter as _;

mod region_containers;
mod table;

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
