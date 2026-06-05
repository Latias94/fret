use super::*;

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
