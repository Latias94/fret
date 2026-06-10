use super::*;

#[test]
fn combo_popup_escape_closes_and_restores_trigger_focus() {
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

    let open = Rc::new(Cell::new(false));
    let opened = Rc::new(Cell::new(false));
    let closed = Rc::new(Cell::new(false));

    let open_out = open.clone();
    let opened_out = opened.clone();
    let closed_out = closed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-generic",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_with_options(
                    "imui-combo-generic-popup",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-generic")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-generic.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                open_out.set(resp.open());
                opened_out.set(resp.opened());
                closed_out.set(resp.closed());
            })
        },
    );
    assert!(!open.get());
    assert!(!opened.get());
    assert!(!closed.get());

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-generic",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);
    let focus_before_open = ui.focus();
    assert!(focus_before_open.is_some());

    app.advance_frame();
    let open_out = open.clone();
    let opened_out = opened.clone();
    let closed_out = closed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-generic",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_with_options(
                    "imui-combo-generic-popup",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-generic")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-generic.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                open_out.set(resp.open());
                opened_out.set(resp.opened());
                closed_out.set(resp.closed());
            })
        },
    );
    assert!(open.get());
    assert!(opened.get());
    assert!(!closed.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-generic.option.0",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let open_out = open.clone();
    let opened_out = opened.clone();
    let closed_out = closed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-generic",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_with_options(
                    "imui-combo-generic-popup",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-generic")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-generic.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                open_out.set(resp.open());
                opened_out.set(resp.opened());
                closed_out.set(resp.closed());
            })
        },
    );
    assert!(!open.get());
    assert!(!opened.get());
    assert!(closed.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-generic.option.0",
    ));
    assert_eq!(ui.focus(), focus_before_open);
}
