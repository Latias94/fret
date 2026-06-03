use super::*;

mod list_box;
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
