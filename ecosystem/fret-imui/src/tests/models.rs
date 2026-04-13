use super::*;
use fret_runtime::KeyChord;

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
        crate::imui(cx, |ui| {
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

#[test]
fn input_text_model_reports_changed_once_after_text_input() {
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

    let model = app.models_mut().insert(String::new());

    let changed = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let changed_out = changed.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert!(text.borrow().is_empty());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "hello");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(changed.get());
    assert_eq!(text.borrow().as_str(), "hello");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.input_text_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert_eq!(text.borrow().as_str(), "hello");
}

#[test]
fn textarea_model_reports_changed_once_after_text_input() {
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

    let model = app.models_mut().insert(String::new());

    let changed = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let changed_out = changed.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert!(text.borrow().is_empty());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(changed.get());
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");
}

#[test]
fn push_id_keeps_changed_signal_stable_after_reorder() {
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

    let model_a = app.models_mut().insert(String::new());
    let model_b = app.models_mut().insert(String::new());

    let order = Rc::new(RefCell::new(vec![1_u8, 2_u8]));
    let changed = Rc::new(RefCell::new(HashMap::<u8, bool>::new()));

    let order_out = order.clone();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-push-id-reorder",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.borrow_mut().clear();
                let order_now = order_out.borrow().clone();
                let changed_map = changed_out.clone();
                ui.column(|ui| {
                    for key in order_now {
                        let model = if key == 1 {
                            model_a.clone()
                        } else {
                            model_b.clone()
                        };
                        let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                        let changed_map = changed_map.clone();
                        ui.push_id(key, |ui| {
                            let resp = ui.input_text_model_with_options(
                                &model,
                                InputTextOptions {
                                    test_id: Some(test_id),
                                    ..Default::default()
                                },
                            );
                            changed_map.borrow_mut().insert(key, resp.changed());
                        });
                    }
                });
            })
        },
    );
    assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), false);
    assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);

    let at = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-input-1");
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "hello");

    app.advance_frame();
    let order_out = order.clone();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-push-id-reorder",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.borrow_mut().clear();
                let order_now = order_out.borrow().clone();
                let changed_map = changed_out.clone();
                ui.column(|ui| {
                    for key in order_now {
                        let model = if key == 1 {
                            model_a.clone()
                        } else {
                            model_b.clone()
                        };
                        let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                        let changed_map = changed_map.clone();
                        ui.push_id(key, |ui| {
                            let resp = ui.input_text_model_with_options(
                                &model,
                                InputTextOptions {
                                    test_id: Some(test_id),
                                    ..Default::default()
                                },
                            );
                            changed_map.borrow_mut().insert(key, resp.changed());
                        });
                    }
                });
            })
        },
    );
    assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), true);
    assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);

    order.borrow_mut().swap(0, 1);
    app.advance_frame();
    let order_out = order.clone();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-push-id-reorder",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.borrow_mut().clear();
                let order_now = order_out.borrow().clone();
                let changed_map = changed_out.clone();
                ui.column(|ui| {
                    for key in order_now {
                        let model = if key == 1 {
                            model_a.clone()
                        } else {
                            model_b.clone()
                        };
                        let test_id: Arc<str> = Arc::from(format!("imui-input-{key}"));
                        let changed_map = changed_map.clone();
                        ui.push_id(key, |ui| {
                            let resp = ui.input_text_model_with_options(
                                &model,
                                InputTextOptions {
                                    test_id: Some(test_id),
                                    ..Default::default()
                                },
                            );
                            changed_map.borrow_mut().insert(key, resp.changed());
                        });
                    }
                });
            })
        },
    );
    assert_eq!(changed.borrow().get(&1).copied().unwrap_or(false), false);
    assert_eq!(changed.borrow().get(&2).copied().unwrap_or(false), false);
}

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
        crate::imui(cx, |ui| {
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

#[test]
fn slider_f32_model_reports_changed_once_after_pointer_input() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(0.0_f32);

    let changed = Rc::new(Cell::new(false));
    let value = Rc::new(Cell::new(0.0_f32));

    let changed_out = changed.clone();
    let value_out = value.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.slider_f32_model_with_options(
                        "Volume",
                        &model,
                        SliderOptions {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            test_id: Some(Arc::from("imui-slider")),
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
    assert!((value.get() - 0.0).abs() <= f32::EPSILON);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let slider = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("imui-slider"))
        .expect("slider semantics node");
    assert_eq!(slider.role, SemanticsRole::Slider);
    assert!(slider.actions.increment);
    assert!(slider.actions.decrement);
    assert!(slider.actions.set_value);
    assert_eq!(slider.extra.numeric.value, Some(0.0));
    assert_eq!(slider.extra.numeric.min, Some(0.0));
    assert_eq!(slider.extra.numeric.max, Some(100.0));
    assert_eq!(slider.extra.numeric.step, Some(1.0));
    assert_eq!(slider.extra.numeric.jump, Some(10.0));

    let slider_node = ui.children(root)[0];
    let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
    let at = Point::new(
        Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.9),
        Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
    );
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
        "imui-slider",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.slider_f32_model_with_options(
                        "Volume",
                        &model,
                        SliderOptions {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            test_id: Some(Arc::from("imui-slider")),
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
    assert!(value.get() >= 70.0);

    app.advance_frame();
    let changed_out = changed.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.slider_f32_model_with_options(
                        "Volume",
                        &model,
                        SliderOptions {
                            min: 0.0,
                            max: 100.0,
                            step: 1.0,
                            test_id: Some(Arc::from("imui-slider")),
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
    assert!(value.get() >= 70.0);
}

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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

#[test]
fn combo_model_popup_escape_closes_and_restores_trigger_focus() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);
    let focus_before_open = ui.focus();
    assert!(focus_before_open.is_some());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape.option.0",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape.option.0",
    ));
    assert_eq!(ui.focus(), focus_before_open);
}

#[test]
fn combo_model_activate_shortcut_is_scoped_to_focused_trigger() {
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
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let target_model = app.models_mut().insert(None::<Arc<str>>);
    let other_model = app.models_mut().insert(None::<Arc<str>>);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        |cx| {
            crate::imui(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );

    let other_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other",
    );
    ui.set_focus(Some(other_node));

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
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        |cx| {
            crate::imui(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target.option.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other.option.0",
    ));

    let target_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target",
    );
    ui.set_focus(Some(target_node));
    assert_eq!(ui.focus(), Some(target_node));

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
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        |cx| {
            crate::imui(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target.option.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other.option.0",
    ));
}

#[test]
fn combo_model_activate_shortcut_repeat_is_opt_in() {
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

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let default_model = app.models_mut().insert(None::<Arc<str>>);
    let repeat_model = app.models_mut().insert(None::<Arc<str>>);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-repeat-default-popup",
                    "Default",
                    &default_model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-repeat.default")),
                        activate_shortcut: Some(default_shortcut),
                        ..Default::default()
                    },
                );
                let _ = ui.combo_model_with_options(
                    "imui-select-repeat-enabled-popup",
                    "Repeat",
                    &repeat_model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-repeat.repeat")),
                        activate_shortcut: Some(repeat_shortcut),
                        shortcut_repeat: true,
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default.option.0",
    ));
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-select-repeat.default.option.0",
        ),
        "expected repeated keydown to leave default combo-model trigger open"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default.option.0",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat.option.0",
    ));
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-select-repeat.repeat.option.0",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}

#[test]
fn combo_model_popup_scope_override_controls_popup_test_id() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-scope",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-popup-scope-override",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-scope",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-scope",
        |cx| {
            crate::imui(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-popup-scope-override",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-select-popup-scope-override",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-select-popup-imui-select-scope",
    ));
}

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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
            crate::imui(cx, |ui| {
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
fn combo_activate_shortcut_is_scoped_to_focused_trigger() {
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
    let target_open = Rc::new(Cell::new(false));
    let other_open = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_open_out: &Rc<Cell<bool>>,
                  other_open_out: &Rc<Cell<bool>>| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                let target = ui.combo_with_options(
                    "imui-combo-shortcut-target-popup",
                    "Target",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-shortcut.target.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let other = ui.combo_with_options(
                    "imui-combo-shortcut-other-popup",
                    "Other",
                    "Beta",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-shortcut.other")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Beta",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-shortcut.other.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                target_open_out.set(target.open());
                other_open_out.set(other.open());
            });
        })
    };

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        |cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(!target_open.get());
    assert!(!other_open.get());

    let other_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.other",
    );
    ui.set_focus(Some(other_node));

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
    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        |cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(!target_open.get());
    assert!(!other_open.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target.option.0",
    ));

    let target_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target",
    );
    ui.set_focus(Some(target_node));
    assert_eq!(ui.focus(), Some(target_node));

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
    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        |cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(target_open.get());
    assert!(!other_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target.option.0",
    ));
}

#[test]
fn combo_activate_shortcut_repeat_is_opt_in() {
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

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_with_options(
                    "imui-combo-repeat-default-popup",
                    "Default",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-repeat.default")),
                        activate_shortcut: Some(default_shortcut),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-repeat.default.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let _ = ui.combo_with_options(
                    "imui-combo-repeat-enabled-popup",
                    "Repeat",
                    "Beta",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-repeat.repeat")),
                        activate_shortcut: Some(repeat_shortcut),
                        shortcut_repeat: true,
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Beta",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-repeat.repeat.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default.option.0",
    ));
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-combo-repeat.default.option.0",
        ),
        "expected repeated keydown to leave default combo trigger open"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default.option.0",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat.option.0",
    ));
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-combo-repeat.repeat.option.0",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}

#[test]
fn combo_can_commit_selection_with_selectable_rows() {
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

    let selected_model = app.models_mut().insert(None::<Arc<str>>);
    let items = ["Alpha", "Beta"];
    let selected = Rc::new(RefCell::new(None::<Arc<str>>));

    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-selectable",
        |cx| {
            crate::imui(cx, |ui| {
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                let preview = current
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("Select..."));
                let current_for_rows = current.clone();
                let model_for_rows = selected_model.clone();
                let _ = ui.combo_with_options(
                    "imui-combo-selectable-popup",
                    "Mode",
                    preview,
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-selectable")),
                        ..Default::default()
                    },
                    move |ui| {
                        for (index, item) in items.iter().enumerate() {
                            let is_selected = current_for_rows
                                .as_ref()
                                .is_some_and(|value| value.as_ref() == *item);
                            let row = ui.selectable_with_options(
                                *item,
                                SelectableOptions {
                                    selected: is_selected,
                                    test_id: Some(Arc::from(format!(
                                        "imui-combo-selectable.option.{index}"
                                    ))),
                                    ..Default::default()
                                },
                            );
                            if row.clicked() {
                                let next = Some(Arc::<str>::from(*item));
                                let _ = ui
                                    .cx_mut()
                                    .app
                                    .models_mut()
                                    .update(&model_for_rows, |value| *value = next.clone());
                                ui.close_popup("imui-combo-selectable-popup");
                            }
                        }
                    },
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(selected.borrow().is_none());

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-selectable",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-selectable",
        |cx| {
            crate::imui(cx, |ui| {
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                let preview = current
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("Select..."));
                let current_for_rows = current.clone();
                let model_for_rows = selected_model.clone();
                let _ = ui.combo_with_options(
                    "imui-combo-selectable-popup",
                    "Mode",
                    preview,
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-selectable")),
                        ..Default::default()
                    },
                    move |ui| {
                        for (index, item) in items.iter().enumerate() {
                            let is_selected = current_for_rows
                                .as_ref()
                                .is_some_and(|value| value.as_ref() == *item);
                            let row = ui.selectable_with_options(
                                *item,
                                SelectableOptions {
                                    selected: is_selected,
                                    test_id: Some(Arc::from(format!(
                                        "imui-combo-selectable.option.{index}"
                                    ))),
                                    ..Default::default()
                                },
                            );
                            if row.clicked() {
                                let next = Some(Arc::<str>::from(*item));
                                let _ = ui
                                    .cx_mut()
                                    .app
                                    .models_mut()
                                    .update(&model_for_rows, |value| *value = next.clone());
                                ui.close_popup("imui-combo-selectable-popup");
                            }
                        }
                    },
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-selectable.option.0",
    ));

    let first_option = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-selectable.option.0",
    );
    click_at(&mut ui, &mut app, &mut services, first_option);

    app.advance_frame();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-selectable",
        |cx| {
            crate::imui(cx, |ui| {
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                let preview = current
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("Select..."));
                let current_for_rows = current.clone();
                let model_for_rows = selected_model.clone();
                let _ = ui.combo_with_options(
                    "imui-combo-selectable-popup",
                    "Mode",
                    preview,
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-selectable")),
                        ..Default::default()
                    },
                    move |ui| {
                        for (index, item) in items.iter().enumerate() {
                            let is_selected = current_for_rows
                                .as_ref()
                                .is_some_and(|value| value.as_ref() == *item);
                            let row = ui.selectable_with_options(
                                *item,
                                SelectableOptions {
                                    selected: is_selected,
                                    test_id: Some(Arc::from(format!(
                                        "imui-combo-selectable.option.{index}"
                                    ))),
                                    ..Default::default()
                                },
                            );
                            if row.clicked() {
                                let next = Some(Arc::<str>::from(*item));
                                let _ = ui
                                    .cx_mut()
                                    .app
                                    .models_mut()
                                    .update(&model_for_rows, |value| *value = next.clone());
                                ui.close_popup("imui-combo-selectable-popup");
                            }
                        }
                    },
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert_eq!(selected.borrow().as_deref(), Some("Alpha"));

    app.advance_frame();
    let selected_out = selected.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-selectable",
        |cx| {
            crate::imui(cx, |ui| {
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                let preview = current
                    .clone()
                    .unwrap_or_else(|| Arc::<str>::from("Select..."));
                let current_for_rows = current.clone();
                let model_for_rows = selected_model.clone();
                let _ = ui.combo_with_options(
                    "imui-combo-selectable-popup",
                    "Mode",
                    preview,
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-selectable")),
                        ..Default::default()
                    },
                    move |ui| {
                        for (index, item) in items.iter().enumerate() {
                            let is_selected = current_for_rows
                                .as_ref()
                                .is_some_and(|value| value.as_ref() == *item);
                            let row = ui.selectable_with_options(
                                *item,
                                SelectableOptions {
                                    selected: is_selected,
                                    test_id: Some(Arc::from(format!(
                                        "imui-combo-selectable.option.{index}"
                                    ))),
                                    ..Default::default()
                                },
                            );
                            if row.clicked() {
                                let next = Some(Arc::<str>::from(*item));
                                let _ = ui
                                    .cx_mut()
                                    .app
                                    .models_mut()
                                    .update(&model_for_rows, |value| *value = next.clone());
                                ui.close_popup("imui-combo-selectable-popup");
                            }
                        }
                    },
                );
                let now = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&selected_model)
                    .unwrap_or(None);
                selected_out.replace(now);
            })
        },
    );
    assert_eq!(selected.borrow().as_deref(), Some("Alpha"));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-selectable.option.0",
    ));
}
