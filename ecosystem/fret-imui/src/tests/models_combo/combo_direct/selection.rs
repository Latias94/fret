use super::*;

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
