use super::*;

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
