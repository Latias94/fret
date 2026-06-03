use super::*;

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
