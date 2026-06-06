use super::*;

#[test]
fn popup_modal_default_outside_press_does_not_close_and_escape_closes() {
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

    let popup_id = "imui-popup-modal-default";
    let modal_test_id = format!("imui-popup-modal-{popup_id}");
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let bootstrap_open = Rc::new(Cell::new(true));

    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let bootstrap_open_out = bootstrap_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui_raw(cx, |ui| {
                if bootstrap_open_out.replace(false) {
                    ui.open_popup(popup_id);
                }
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(8.0), Px(8.0)),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui_raw(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-default-outside",
        |cx| {
            crate::imui_raw(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));
}

#[test]
fn popup_modal_can_close_on_outside_press_when_enabled() {
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

    let popup_id = "imui-popup-modal-outside-close";
    let modal_test_id = format!("imui-popup-modal-{popup_id}");
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let bootstrap_open = Rc::new(Cell::new(true));

    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let bootstrap_open_out = bootstrap_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                if bootstrap_open_out.replace(false) {
                    ui.open_popup(popup_id);
                }
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(opened.get());
    assert!(open_state.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));

    click_at(
        &mut ui,
        &mut app,
        &mut services,
        Point::new(Px(8.0), Px(8.0)),
    );

    app.advance_frame();
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-modal-outside-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                opened_out.set(ui.begin_popup_modal_with_options(
                    popup_id,
                    None,
                    PopupModalOptions {
                        size: Size::new(Px(160.0), Px(96.0)),
                        close_on_outside_press: true,
                    },
                    |ui| {
                        ui.text("Modal");
                    },
                ));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        modal_test_id.as_str(),
    ));
}
