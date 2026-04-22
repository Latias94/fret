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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let target_value = Rc::new(Cell::new(false));
    let other_value = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_value_out: &Rc<Cell<bool>>,
                  other_value_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
            crate::imui_raw(cx, |ui| {
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
fn slider_lifecycle_reports_edit_and_deactivated_after_pointer_commit() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(0.0_f32);
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let value = Rc::new(Cell::new(0.0_f32));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  value_out: &Rc<Cell<f32>>| {
        crate::imui_raw(cx, |ui| {
            let resp = ui.slider_f32_model_with_options(
                "Volume",
                &model,
                SliderOptions {
                    min: 0.0,
                    max: 100.0,
                    step: 1.0,
                    test_id: Some(Arc::from("imui-slider-lifecycle")),
                    ..Default::default()
                },
            );
            activated_out.set(resp.activated());
            deactivated_out.set(resp.deactivated());
            edited_out.set(resp.edited());
            after_edit_out.set(resp.deactivated_after_edit());
            let now = ui
                .cx_mut()
                .app
                .models()
                .get_copied(&model)
                .unwrap_or_default();
            value_out.set(now);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert_eq!(value.get(), 0.0);

    let slider_node = ui.children(root)[0];
    let slider_bounds = ui.debug_node_bounds(slider_node).expect("slider bounds");
    let start = Point::new(
        Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.1),
        Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
    );
    let drag = Point::new(
        Px(slider_bounds.origin.x.0 + slider_bounds.size.width.0 * 0.9),
        Px(slider_bounds.origin.y.0 + slider_bounds.size.height.0 * 0.5),
    );
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert!(value.get() > 0.0);

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        drag,
        MouseButtons {
            left: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert!(value.get() >= 70.0);

    pointer_up_at(&mut ui, &mut app, &mut services, drag);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let value_out = value.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-slider-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &value_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert!(value.get() > 0.0);
}
