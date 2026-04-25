use super::*;
use fret_ui_kit::imui::ButtonOptions;
use fret_ui_kit::imui::InputTextMode;
use fret_ui_kit::imui::TextAreaOptions;

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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
fn input_text_password_mode_obscures_paint_text_without_mutating_model() {
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

    let model = app.models_mut().insert(String::from("secret"));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-password",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        mode: InputTextMode::Password,
                        test_id: Some(Arc::from("imui-input-text-password")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    services.prepared.clear();
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        services.prepared.iter().any(|text| text == "••••••"),
        "expected password mode to paint an obscured string"
    );
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("secret"),
        "expected password mode to preserve the underlying model value"
    );
}

#[test]
fn input_text_focus_keeps_control_bounds_stable() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-stable-bounds")),
                        ..Default::default()
                    },
                );
                let _ = ui.button_with_options(
                    "Sibling",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-input-text-stable-bounds.sibling")),
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
        "imui-input-text-stable-bounds",
        |cx| render(cx),
    );
    let input_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-stable-bounds",
    );
    let before = ui.debug_node_bounds(input_node).expect("input bounds");

    let at = Point::new(
        Px(before.origin.x.0 + before.size.width.0 * 0.5),
        Px(before.origin.y.0 + before.size.height.0 * 0.5),
    );
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-stable-bounds",
        |cx| render(cx),
    );
    let input_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-stable-bounds",
    );
    let after = ui.debug_node_bounds(input_node).expect("input bounds");

    assert_eq!(after.origin, before.origin);
    assert_eq!(after.size, before.size);
}

#[test]
fn input_text_lifecycle_tracks_focus_edit_and_blur_edges() {
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
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  text_out: &Rc<RefCell<String>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let resp = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());

                let _ = ui.button_with_options(
                    "Blur target",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-input-text-lifecycle.blur-target")),
                        ..Default::default()
                    },
                );
            });

            let current = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&model)
                .unwrap_or_default();
            text_out.replace(current);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-lifecycle",
    );
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    text_input_event(&mut ui, &mut app, &mut services, "hello");

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello");

    let blur_target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-lifecycle.blur-target",
    );
    click_at(&mut ui, &mut app, &mut services, blur_target);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
fn textarea_lifecycle_tracks_focus_edit_and_blur_edges() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  text_out: &Rc<RefCell<String>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let resp = ui.textarea_model_with_options(
                    &model,
                    TextAreaOptions {
                        test_id: Some(Arc::from("imui-textarea-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());

                let _ = ui.button_with_options(
                    "Blur target",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-textarea-lifecycle.blur-target")),
                        ..Default::default()
                    },
                );
            });

            let current = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&model)
                .unwrap_or_default();
            text_out.replace(current);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    let input = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    text_input_event(&mut ui, &mut app, &mut services, "hello\nworld");

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello\nworld");

    let blur_target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-textarea-lifecycle.blur-target",
    );
    click_at(&mut ui, &mut app, &mut services, blur_target);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello\nworld");
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
