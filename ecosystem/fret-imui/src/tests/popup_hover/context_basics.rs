use super::*;

#[test]
fn context_menu_popup_opens_on_right_click_and_closes_on_outside_click() {
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

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(open.get());

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(230.0), Px(110.0)),
    );

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
            })
        },
    );
    assert!(!open.get());
}

#[test]
fn context_menu_popup_closes_if_trigger_disappears_for_a_frame() {
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

    let open = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(!open.get());
    assert!(!open_state.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |ui| {
                    ui.text("Menu");
                }));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(open.get());
    assert!(open_state.get());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.text("Trigger disappeared");
            })
        },
    );

    app.advance_frame();
    let open_out = open.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-disappear",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
                let model = ui.popup_open_model("ctx");
                open_state_out.set(ui.cx_mut().app.models().get_copied(&model).unwrap_or(false));
            })
        },
    );
    assert!(!open.get());
    assert!(!open_state.get());
}
