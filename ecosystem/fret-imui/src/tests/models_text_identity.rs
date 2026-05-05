use super::*;
use fret_ui_kit::imui::InputTextOptions;

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
