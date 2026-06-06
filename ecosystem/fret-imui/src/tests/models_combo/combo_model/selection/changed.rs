use super::*;

#[test]
fn combo_model_reports_changed_once_after_option_pick() {
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

    let changed = Rc::new(Cell::new(false));
    let selected = Rc::new(RefCell::new(None::<Arc<str>>));

    let changed_out = changed.clone();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.combo_model_with_options(
                        "imui-select-popup",
                        "Mode",
                        &model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(!changed.get());
    assert!(selected.borrow().is_none());

    let trigger = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-select");
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let changed_out = changed.clone();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.combo_model_with_options(
                        "imui-select-popup",
                        "Mode",
                        &model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(!changed.get());
    assert!(selected.borrow().is_none());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select.option.0",
    ));

    let first_option = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select.option.0",
    );
    click_at(&mut ui, &mut app, &mut services, first_option);

    app.advance_frame();
    let changed_out = changed.clone();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.combo_model_with_options(
                        "imui-select-popup",
                        "Mode",
                        &model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(changed.get());
    assert_eq!(selected.borrow().as_deref(), Some("Alpha"));
    app.advance_frame();
    let changed_out = changed.clone();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.combo_model_with_options(
                        "imui-select-popup",
                        "Mode",
                        &model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui.cx_mut().app.models().get_cloned(&model).unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(!changed.get());
    assert_eq!(selected.borrow().as_deref(), Some("Alpha"));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select.option.0",
    ));
}
