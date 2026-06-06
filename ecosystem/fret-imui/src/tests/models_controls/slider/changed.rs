use super::*;

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
