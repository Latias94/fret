use super::*;

#[test]
fn floating_window_closable_false_hides_close_button_and_escape_does_not_close() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let open = app.models_mut().insert(true);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.close:demo",
        ),
        "expected close button to be hidden when closable=false"
    );

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
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

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        app.models().get_copied(&open).unwrap_or(false),
        "expected Escape not to close when closable=false"
    );
}

#[test]
fn floating_window_movable_false_does_not_move_when_dragging_title_bar() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let position = Rc::new(Cell::new(Point::default()));

    let position_out = position.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-movable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    window_behavior_options(FloatingWindowOptions {
                        movable: false,
                        ..Default::default()
                    }),
                    |ui| ui.text("Hello"),
                );
                position_out.set(resp.position());
            })
        },
    );
    let _ = ui.children(root);
    let before = position.get();

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, title_bar);
    let moved = Point::new(Px(title_bar.x.0 + 30.0), Px(title_bar.y.0 + 8.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );
    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);

    app.advance_frame();
    let position_out = position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-movable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    window_behavior_options(FloatingWindowOptions {
                        movable: false,
                        ..Default::default()
                    }),
                    |ui| ui.text("Hello"),
                );
                position_out.set(resp.position());
            })
        },
    );

    assert_eq!(
        position.get(),
        before,
        "expected window position unchanged when movable=false"
    );
}

#[test]
fn floating_window_resizable_false_hides_resize_handles() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resizable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            resizable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles hidden when resizable=false"
    );
}

#[test]
fn floating_window_collapsible_false_does_not_toggle_on_title_double_click() {
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

    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );
    assert!(!collapsed.get());

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );

    assert!(
        !collapsed.get(),
        "expected title-bar double click not to toggle collapse when collapsible=false"
    );
}
