use super::*;

#[test]
fn checkbox_changed_is_delivered_once_and_updates_model() {
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
        "imui-checkbox",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.checkbox_model("Enabled", &model).changed());
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
        "imui-checkbox",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.checkbox_model("Enabled", &model).changed());
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
        "imui-checkbox",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.checkbox_model("Enabled", &model).changed());
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

#[test]
fn checkbox_model_activate_shortcut_is_scoped_to_focused_checkbox() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let target_model = app.models_mut().insert(false);
    let other_model = app.models_mut().insert(false);
    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let target_value = Rc::new(Cell::new(false));
    let other_value = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_value_out: &Rc<Cell<bool>>,
                  other_value_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.checkbox_model_with_options(
                    "Target",
                    &target_model,
                    CheckboxOptions {
                        test_id: Some(Arc::from("imui-checkbox-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                );
                let _ = ui.checkbox_model_with_options(
                    "Other",
                    &other_model,
                    CheckboxOptions {
                        test_id: Some(Arc::from("imui-checkbox-shortcut.other")),
                        ..Default::default()
                    },
                );
            });

            let target_now = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&target_model)
                .unwrap_or_default();
            let other_now = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&other_model)
                .unwrap_or_default();
            target_value_out.set(target_now);
            other_value_out.set(other_now);
        })
    };

    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(!target_value.get());
    assert!(!other_value.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(
        !target_value.get() && !other_value.get(),
        "expected unfocused checkbox shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-checkbox-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(!target_value.get());
    assert!(other_value.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(
        !target_value.get() && other_value.get(),
        "expected shortcut on another focused checkbox to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-checkbox-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(target_value.get());
    assert!(other_value.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let target_value_out = target_value.clone();
    let other_value_out = other_value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(!target_value.get());
    assert!(other_value.get());
}
