use super::*;

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
