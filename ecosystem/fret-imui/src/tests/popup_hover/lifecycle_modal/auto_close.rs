use super::*;

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
