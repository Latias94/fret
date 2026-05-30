use super::*;
use fret_ui_kit::imui::RadioOptions;

#[test]
fn radio_clicked_is_delivered_once() {
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

    let clicked = Rc::new(Cell::new(false));

    let clicked_out = clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.radio("Choice", false).clicked());
            })
        },
    );
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.radio("Choice", false).clicked());
            })
        },
    );
    assert!(clicked.get());

    app.advance_frame();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-clicked",
        |cx| {
            crate::imui_raw(cx, |ui| {
                clicked_out.set(ui.radio("Choice", false).clicked());
            })
        },
    );
    assert!(!clicked.get());
}

#[test]
fn radio_activate_shortcut_is_scoped_to_focused_radio() {
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

    let shortcut = ctrl_shortcut(KeyCode::KeyK);
    let target_clicked = Rc::new(Cell::new(false));
    let other_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_clicked_out: &Rc<Cell<bool>>,
                  other_clicked_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let target = ui.radio_with_options(
                    "Target",
                    false,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-radio-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                );
                target_clicked_out.set(target.clicked());

                let other = ui.radio_with_options(
                    "Other",
                    false,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-radio-shortcut.other")),
                        ..Default::default()
                    },
                );
                other_clicked_out.set(other.clicked());
            });
        })
    };

    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected unfocused radio shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-radio-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected shortcut on another focused radio to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-radio-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());
}

#[test]
fn radio_shift_f10_sets_context_menu_requested_true_once() {
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

    let requested = Rc::new(Cell::new(false));
    let render = |cx: &mut ElementContext<'_, TestHost>, requested_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.radio_with_options(
                "Choice",
                false,
                RadioOptions {
                    test_id: Some(Arc::from("imui-radio-context.choice")),
                    ..Default::default()
                },
            );
            requested_out.set(response.context_menu_requested());
        })
    };

    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-context",
        |cx| render(cx, &requested_out),
    );
    assert!(!requested.get());

    let radio = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-radio-context.choice",
    );
    click_at(&mut ui, &mut app, &mut services, radio);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-context",
        |cx| render(cx, &requested_out),
    );
    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-context",
        |cx| render(cx, &requested_out),
    );
    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-radio-context",
        |cx| render(cx, &requested_out),
    );
    assert!(!requested.get());
}
