use super::*;

#[test]
fn switch_model_activate_shortcut_is_scoped_to_focused_switch() {
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
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let target_value = Rc::new(Cell::new(false));
    let other_value = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_value_out: &Rc<Cell<bool>>,
                  other_value_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.switch_model_with_options(
                    "Target",
                    &target_model,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-switch-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Other",
                    &other_model,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-switch-shortcut.other")),
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
        "imui-switch-shortcut",
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
        "imui-switch-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(
        !target_value.get() && !other_value.get(),
        "expected unfocused switch shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-switch-shortcut.other",
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
        "imui-switch-shortcut",
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
        "imui-switch-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(
        !target_value.get() && other_value.get(),
        "expected shortcut on another focused switch to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-switch-shortcut.target",
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
        "imui-switch-shortcut",
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
        "imui-switch-shortcut",
        |cx| render(cx, &target_value_out, &other_value_out),
    );
    assert!(!target_value.get());
    assert!(other_value.get());
}
