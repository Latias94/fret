use super::*;

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
