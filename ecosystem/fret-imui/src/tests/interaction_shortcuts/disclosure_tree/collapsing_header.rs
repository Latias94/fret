use super::*;

#[test]
fn collapsing_header_activate_shortcut_is_scoped_to_focused_trigger() {
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

    let target_open = app.models_mut().insert(false);
    let other_open = app.models_mut().insert(false);
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let target_state = Rc::new(Cell::new(false));
    let other_state = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_state_out: &Rc<Cell<bool>>,
                  other_state_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.collapsing_header_with_options(
                    "target",
                    "Target",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(target_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                    |_ui| {},
                );
                let _ = ui.collapsing_header_with_options(
                    "other",
                    "Other",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(other_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.other")),
                        ..Default::default()
                    },
                    |_ui| {},
                );
            });

            target_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&target_open)
                    .unwrap_or_default(),
            );
            other_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&other_open)
                    .unwrap_or_default(),
            );
        })
    };

    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(!other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && !other_state.get(),
        "expected unfocused disclosure shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && other_state.get(),
        "expected shortcut on another disclosure trigger to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(target_state.get());
    assert!(other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());
}
