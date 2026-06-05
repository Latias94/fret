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

#[test]
fn combo_lifecycle_tracks_open_session_edges() {
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

    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let open = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  open_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            let resp = ui.combo_with_options(
                "imui-combo-lifecycle-popup",
                "Mode",
                "Alpha",
                ComboOptions {
                    test_id: Some(Arc::from("imui-combo-lifecycle")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.selectable_with_options(
                        "Alpha",
                        SelectableOptions {
                            test_id: Some(Arc::from("imui-combo-lifecycle.option.0")),
                            ..Default::default()
                        },
                    );
                },
            );
            activated_out.set(resp.response().activated());
            deactivated_out.set(resp.response().deactivated());
            edited_out.set(resp.response().edited());
            after_edit_out.set(resp.response().deactivated_after_edit());
            open_out.set(resp.open());
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &open_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(!open.get());

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-lifecycle",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &open_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(open.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &open_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(!open.get());
}
