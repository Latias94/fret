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

#[test]
fn combo_model_lifecycle_reports_edit_on_option_pick() {
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

    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let open = Rc::new(Cell::new(false));

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
        "imui-select-lifecycle",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_model_with_options(
                    "imui-select-lifecycle-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());
                let popup_open = ui.popup_open_model("imui-select-lifecycle-popup");
                let open_now = ui
                    .cx_mut()
                    .read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false);
                open_out.set(open_now);
            })
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
        "imui-select-lifecycle",
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
        "imui-select-lifecycle",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_model_with_options(
                    "imui-select-lifecycle-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());
                let popup_open = ui.popup_open_model("imui-select-lifecycle-popup");
                let open_now = ui
                    .cx_mut()
                    .read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false);
                open_out.set(open_now);
            })
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(open.get());

    let first_option = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-lifecycle.option.0",
    );
    click_at(&mut ui, &mut app, &mut services, first_option);

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
        "imui-select-lifecycle",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.combo_model_with_options(
                    "imui-select-lifecycle-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());
                let popup_open = ui.popup_open_model("imui-select-lifecycle-popup");
                let open_now = ui
                    .cx_mut()
                    .read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false);
                open_out.set(open_now);
            })
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(edited.get());
    assert!(after_edit.get());
    assert!(!open.get());
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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

    let _other_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        &|cx| {
            crate::imui_raw(cx, |ui| {
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

    let _target_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        &|cx| {
            crate::imui_raw(cx, |ui| {
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
        crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            activated_out.set(resp.trigger.activated());
            deactivated_out.set(resp.trigger.deactivated());
            edited_out.set(resp.trigger.edited());
            after_edit_out.set(resp.trigger.deactivated_after_edit());
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
        crate::imui_raw(cx, |ui| {
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

    let _other_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.other",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        &|cx| render(cx, &target_open_out, &other_open_out),
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

    let _target_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        &|cx| render(cx, &target_open_out, &other_open_out),
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
        crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
