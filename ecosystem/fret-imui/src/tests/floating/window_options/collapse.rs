use super::*;

#[test]
fn floating_window_title_bar_double_click_toggles_collapsed() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let collapsed = Rc::new(Cell::new(false));
    let resizing = Rc::new(Cell::new(false));
    let area_id = Rc::new(Cell::new(0u64));

    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.id().0);
            })
        },
    );
    let _ = ui.children(root);
    assert!(!collapsed.get());
    assert!(!resizing.get());
    let area_id_before = area_id.get();
    assert_ne!(area_id_before, 0, "expected non-zero floating area id");

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let title_bar_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    let title_bar_bounds = ui
        .debug_node_bounds(title_bar_node)
        .expect("title bar bounds");
    let title_bar = Point::new(
        Px(title_bar_bounds.origin.x.0 + title_bar_bounds.size.width.0 * 0.5),
        Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 * 0.5),
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.id().0);
            })
        },
    );
    assert!(collapsed.get());
    assert!(!resizing.get());
    let area_id_collapsed = area_id.get();
    assert_eq!(
        area_id_collapsed, area_id_before,
        "expected floating area id stable across collapse"
    );

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let collapsed_bounds = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        collapsed_bounds.size.height.0 < before.size.height.0,
        "expected collapsed window to be shorter"
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles hidden while collapsed"
    );

    let title_bar_after_collapse_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    let title_bar_after_collapse_bounds = ui
        .debug_node_bounds(title_bar_after_collapse_node)
        .expect("title bar bounds");
    let title_bar_after_collapse = Point::new(
        Px(title_bar_after_collapse_bounds.origin.x.0
            + title_bar_after_collapse_bounds.size.width.0 * 0.5),
        Px(title_bar_after_collapse_bounds.origin.y.0
            + title_bar_after_collapse_bounds.size.height.0 * 0.5),
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar_after_collapse);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.id().0);
            })
        },
    );
    assert!(!collapsed.get());
    assert!(!resizing.get());
    assert_eq!(
        area_id.get(),
        area_id_before,
        "expected floating area id stable across expand"
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles restored after expanding"
    );
}
