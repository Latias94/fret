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
fn toggle_model_reports_changed_once_after_click() {
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
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-toggle",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.toggle_model_with_options(
                        "Flag",
                        &model,
                        ToggleOptions {
                            test_id: Some(Arc::from("imui-toggle")),
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

    let at = point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-toggle");
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
        "imui-toggle",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.toggle_model_with_options(
                        "Flag",
                        &model,
                        ToggleOptions {
                            test_id: Some(Arc::from("imui-toggle")),
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
        "imui-toggle",
        |cx| {
            crate::imui(cx, |ui| {
                changed_out.set(
                    ui.toggle_model_with_options(
                        "Flag",
                        &model,
                        ToggleOptions {
                            test_id: Some(Arc::from("imui-toggle")),
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
fn select_model_reports_changed_once_after_option_pick() {
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
                    ui.select_model_with_options(
                        "Mode",
                        &model,
                        &items,
                        SelectOptions {
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
                    ui.select_model_with_options(
                        "Mode",
                        &model,
                        &items,
                        SelectOptions {
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
                    ui.select_model_with_options(
                        "Mode",
                        &model,
                        &items,
                        SelectOptions {
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
                    ui.select_model_with_options(
                        "Mode",
                        &model,
                        &items,
                        SelectOptions {
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
fn select_popup_escape_closes_and_restores_trigger_focus() {
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
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
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
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
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
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
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
fn select_popup_scope_override_controls_popup_test_id() {
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
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        popup_scope_id: Some(Arc::from("imui-select-popup-scope-override")),
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
                let _ = ui.select_model_with_options(
                    "Mode",
                    &model,
                    &items,
                    SelectOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        popup_scope_id: Some(Arc::from("imui-select-popup-scope-override")),
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
