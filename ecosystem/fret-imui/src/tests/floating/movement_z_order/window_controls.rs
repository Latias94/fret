use super::*;

#[test]
fn window_wrapper_reports_position_and_size() {
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

    let reported_pos = Rc::new(Cell::new(Point::new(Px(0.0), Px(0.0))));
    let reported_size = Rc::new(Cell::new(None::<Size>));

    let reported_pos_out = reported_pos.clone();
    let reported_size_out = reported_size.clone();

    let initial_position = Point::new(Px(10.0), Px(10.0));
    let initial_size = Size::new(Px(140.0), Px(80.0));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-window-wrapper-reports-position-and-size",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    initial_position,
                    resizable_window_options(initial_size),
                    |ui| ui.text("Hello"),
                );
                reported_pos_out.set(resp.position());
                reported_size_out.set(resp.size());
            })
        },
    );

    assert_eq!(reported_pos.get(), initial_position);
    assert_eq!(reported_size.get(), Some(initial_size));
}

#[test]
fn floating_window_close_button_sets_open_false() {
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

    let open = app.models_mut().insert(true);

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options(&open),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let _ = floating_window_nodes(&ui, root);
    let close = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.close:demo",
    );
    click_at(&mut ui, &mut app, &mut services, close);
    assert!(!app.models().get_copied(&open).unwrap_or(true));
}

#[test]
fn floating_window_escape_sets_open_false_after_focusing_title_bar() {
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

    let open = app.models_mut().insert(true);

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-escape",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options(&open),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let _ = floating_window_nodes(&ui, root);
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
    click_at(&mut ui, &mut app, &mut services, title_bar);
    assert!(ui.focus().is_some(), "expected title bar to take focus");

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );
    assert!(!app.models().get_copied(&open).unwrap_or(true));
}
