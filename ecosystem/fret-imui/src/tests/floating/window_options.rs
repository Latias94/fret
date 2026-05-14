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

#[test]
fn floating_window_resizes_when_dragging_corner_handle() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let corner = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.resize.corner:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, corner);
    let moved = Point::new(Px(corner.x.0 + 20.0), Px(corner.y.0 + 10.0));
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

    app.advance_frame();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );
    let _ = ui.children(root);

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let after = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        after.size.width.0 > before.size.width.0,
        "expected window to grow wider"
    );
    assert!(
        after.size.height.0 > before.size.height.0,
        "expected window to grow taller"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
}

#[test]
fn floating_window_resizes_from_left_updates_origin_and_width() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize-left",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(80.0), Px(40.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    let _ = ui.children(root);
    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let left = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.resize.left:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, left);
    let moved = Point::new(Px(left.x.0 - 18.0), left.y);
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

    app.advance_frame();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize-left",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(80.0), Px(40.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );
    let _ = ui.children(root);

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let after = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected origin.x to move left when resizing from left"
    );
    assert!(
        after.size.width.0 > before.size.width.0,
        "expected width to grow when resizing from left"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
}

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
