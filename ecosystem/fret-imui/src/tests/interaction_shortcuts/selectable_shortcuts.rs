use super::*;

#[test]
fn selectable_activate_shortcut_is_scoped_to_focused_item() {
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
                target_clicked_out.set(
                    ui.selectable_with_options(
                        "Target",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
                other_clicked_out.set(
                    ui.selectable_with_options(
                        "Other",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.other")),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
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
        "imui-selectable-shortcut",
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
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected unfocused shortcut to stay local to the selectable"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.other",
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
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());

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
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected shortcut on another focused selectable to do nothing"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.target",
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
        "imui-selectable-shortcut",
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
        "imui-selectable-shortcut",
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
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());
}
#[test]
fn selectable_activate_shortcut_preserves_popup_arrow_nav() {
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

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let popup_id = "imui-selectable-shortcut-popup";

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let popup_open = ui.popup_open_model(popup_id);
            let is_open = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&popup_open)
                .unwrap_or(false);
            if !is_open {
                ui.open_popup_at(
                    popup_id,
                    Rect::new(Point::new(Px(48.0), Px(48.0)), Size::new(Px(1.0), Px(1.0))),
                );
            }

            assert!(ui.begin_popup_menu_with_options(
                popup_id,
                None,
                fret_ui_kit::imui::PopupMenuOptions {
                    estimated_size: Size::new(Px(160.0), Px(90.0)),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.selectable_with_options(
                        "Alpha",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.alpha")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.selectable_with_options(
                        "Beta",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.beta")),
                            ..Default::default()
                        },
                    );
                },
            ));
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    let alpha = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut-popup.alpha",
    );
    click_at(&mut ui, &mut app, &mut services, alpha);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.alpha")
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.beta")
    );
}
