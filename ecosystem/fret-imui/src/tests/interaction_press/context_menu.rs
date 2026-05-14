use super::*;

#[test]
fn right_click_sets_context_menu_requested_true_once() {
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

    let requested = Rc::new(Cell::new(false));
    let secondary_clicked = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(!requested.get());
    assert!(!secondary_clicked.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(requested.get());
    assert!(secondary_clicked.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let secondary_clicked_out = secondary_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-right-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                requested_out.set(resp.context_menu_requested());
                secondary_clicked_out.set(resp.secondary_clicked());
            })
        },
    );
    assert!(!requested.get());
    assert!(!secondary_clicked.get());
}
#[test]
fn double_click_sets_double_clicked_true_once() {
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

    let double_clicked = Rc::new(Cell::new(false));
    let double_clicked_out = double_clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(!double_clicked.get());

    let at = first_child_point(&ui, root);
    double_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let double_clicked_out = double_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(double_clicked.get());

    app.advance_frame();
    let double_clicked_out = double_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-double-click",
        |cx| {
            crate::imui_raw(cx, |ui| {
                double_clicked_out.set(ui.button("OK").double_clicked());
            })
        },
    );
    assert!(!double_clicked.get());
}
#[test]
fn shift_f10_sets_context_menu_requested_true_once() {
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

    let requested = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(ui.button("OK").context_menu_requested());
            })
        },
    );
    assert!(!requested.get());
}
