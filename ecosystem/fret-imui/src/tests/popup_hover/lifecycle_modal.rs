use super::*;

#[test]
fn drop_popup_scope_closes_and_forgets_internal_state() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(None::<Arc<str>>);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let popup_scope_id: Arc<str> = Arc::from("imui-drop-popup-scope");

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drop-popup-trigger",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drop-popup-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.drop_popup_scope(popup_scope_id.as_ref());
                let _ = ui.combo_model_with_options(
                    popup_scope_id.as_ref(),
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-drop-popup-trigger")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-drop-popup-scope",
    ));
}

#[test]
fn popup_closes_after_one_frame_without_keep_alive() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let popup_id = "imui-popup-auto-close";
    let anchor = Rect::new(Point::new(Px(12.0), Px(12.0)), Size::new(Px(1.0), Px(1.0)));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.open_popup_at(popup_id, anchor);
                // Intentionally do not call `begin_popup_menu*` this frame.
            })
        },
    );

    app.advance_frame();
    let open_state = Rc::new(Cell::new(false));
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(open_state.get());

    app.advance_frame();
    let opened = Rc::new(Cell::new(false));
    let open_state = Rc::new(Cell::new(false));
    let opened_out = opened.clone();
    let open_state_out = open_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-auto-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                opened_out.set(ui.begin_popup_menu(popup_id, None, |_ui| {}));
                let open = ui.popup_open_model(popup_id);
                open_state_out.set(ui.cx_mut().app.models().get_copied(&open).unwrap_or(false));
            })
        },
    );

    assert!(!opened.get());
    assert!(!open_state.get());
}

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
