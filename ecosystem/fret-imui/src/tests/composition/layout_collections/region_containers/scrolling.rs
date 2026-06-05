use super::*;

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
