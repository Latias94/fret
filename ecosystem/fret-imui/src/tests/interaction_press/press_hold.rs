use super::*;

#[allow(dead_code)]
#[test]
fn holding_press_does_not_repeat_clicked() {
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

    let clicked = Rc::new(Cell::new(false));
    let clicked_out = clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());

    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(clicked.get());

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hold-press",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.button("OK").clicked());
            })
        },
    );
    assert!(!clicked.get());
}
#[test]
fn long_press_sets_long_pressed_true_once_and_reports_holding() {
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

    let long_pressed = Rc::new(Cell::new(false));
    let holding = Rc::new(Cell::new(false));

    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(!holding.get());

    let at = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, at);
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0);

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );

    assert!(long_pressed.get());
    assert!(holding.get());

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(holding.get());

    pointer_up_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let long_pressed_out = long_pressed.clone();
    let holding_out = holding.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-long-press-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                long_pressed_out.set(resp.long_pressed());
                holding_out.set(resp.press_holding());
            })
        },
    );
    assert!(!long_pressed.get());
    assert!(!holding.get());
}
