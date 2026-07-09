use super::*;

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
            render,
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
