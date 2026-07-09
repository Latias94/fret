use super::*;

#[test]
fn label_identity_model_controls_hide_suffixes_and_preserve_focus_across_reorder() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(300.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let flipped = Rc::new(Cell::new(false));
    let checkbox_a = app.models_mut().insert(false);
    let checkbox_b = app.models_mut().insert(false);
    let switch = app.models_mut().insert(false);
    let slider = app.models_mut().insert(0.25f32);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let row_a = (
                    String::from("Check A###check-a"),
                    checkbox_a.clone(),
                    "imui-label-identity.checkbox.a",
                );
                let row_b = (
                    String::from("Check B###check-b"),
                    checkbox_b.clone(),
                    "imui-label-identity.checkbox.b",
                );
                let rows = if flipped.get() {
                    vec![row_b, row_a]
                } else {
                    vec![row_a, row_b]
                };
                for (label, model, test_id) in rows {
                    let _ = ui.checkbox_model_with_options(
                        label,
                        &model,
                        CheckboxOptions {
                            test_id: Some(Arc::from(test_id)),
                            ..Default::default()
                        },
                    );
                }

                let _ = ui.radio_with_options(
                    "Radio###radio-stable",
                    false,
                    RadioOptions {
                        test_id: Some(Arc::from("imui-label-identity.radio")),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Switch##switch-id",
                    &switch,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-label-identity.switch")),
                        ..Default::default()
                    },
                );
                let _ = ui.slider_f32_model_with_options(
                    "Amount##slider-id",
                    &slider,
                    SliderOptions {
                        test_id: Some(Arc::from("imui-label-identity.slider")),
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
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
        "imui-label-identity-model-controls",
        |cx| render(cx),
    );

    let _checkbox_a = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.checkbox.a",
    );

    flipped.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-model-controls",
        &render,
    );

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-label-identity.checkbox.a"))
    );
    assert!(services.prepared.iter().any(|text| text == "Check A"));
    assert!(services.prepared.iter().any(|text| text == "Radio"));
    assert!(services.prepared.iter().any(|text| text == "Switch"));
    assert!(services.prepared.iter().any(|text| text == "Amount"));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("check-a")
            || text.contains("check-b")
            || text.contains("radio-stable")
            || text.contains("switch-id")
            || text.contains("slider-id")),
        "label identity suffixes should not be painted: {:?}",
        services.prepared
    );
}
