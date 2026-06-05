use super::*;

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
