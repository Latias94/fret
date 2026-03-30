use super::*;

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
                use fret_ui_kit::imui::UiWriterUiKitExt as _;

                let builder = fret_ui_kit::ui::text("Hello").text_sm();
                ui.add_ui(builder);
            })
        },
    );

    assert_eq!(ui.children(root).len(), 1);
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
            crate::imui(cx, |ui| {
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
// Note: `for_each_keyed` is exercised indirectly by downstream ecosystem crates. The core
// smoke tests above focus on interaction correctness (`clicked` / `changed`).
