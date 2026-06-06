use super::*;

#[test]
fn switch_model_reports_changed_once_after_click() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(false);

    let changed = Rc::new(Cell::new(false));
    let value = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let value_out = value.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-switch",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.switch_model_with_options(
                        "Power",
                        &model,
                        SwitchOptions {
                            test_id: Some(Arc::from("imui-switch")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_copied(&model)
                    .unwrap_or_default();
                value_out.set(now);
            })
        },
    );
    assert!(!changed.get());
    assert!(!value.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let changed_out = changed.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-switch",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.switch_model_with_options(
                        "Power",
                        &model,
                        SwitchOptions {
                            test_id: Some(Arc::from("imui-switch")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_copied(&model)
                    .unwrap_or_default();
                value_out.set(now);
            })
        },
    );
    assert!(changed.get());
    assert!(value.get());

    app.advance_frame();
    let changed_out = changed.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-switch",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.switch_model_with_options(
                        "Power",
                        &model,
                        SwitchOptions {
                            test_id: Some(Arc::from("imui-switch")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_copied(&model)
                    .unwrap_or_default();
                value_out.set(now);
            })
        },
    );
    assert!(!changed.get());
    assert!(value.get());
}
