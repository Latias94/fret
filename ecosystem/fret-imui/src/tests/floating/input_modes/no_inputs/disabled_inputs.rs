use super::*;

#[test]
fn floating_window_inputs_enabled_false_blocks_child_pressables() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(200.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let clicked_model = app.models_mut().insert(false);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-no-inputs",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.window_with_options(
                        "demo",
                        "Demo",
                        Point::new(Px(10.0), Px(10.0)),
                        window_behavior_options(fret_ui_kit::imui::FloatingWindowOptions {
                            inputs_enabled: false,
                            ..Default::default()
                        }),
                        |ui| {
                            let model = clicked_model.clone();
                            let element = ui.cx_mut().pressable(
                                {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(24.0));
                                    props.a11y = fret_ui::element::PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Blocked")),
                                        test_id: Some(Arc::from(
                                            "imui-test.float_window.inputs_enabled_false.pressable",
                                        )),
                                        ..Default::default()
                                    };
                                    props
                                },
                                move |cx, _state| {
                                    cx.pressable_on_activate(Arc::new(
                                        move |host, acx, _reason| {
                                            let _ = host
                                                .models_mut()
                                                .update(&model, |v: &mut bool| *v = true);
                                            host.notify(acx);
                                        },
                                    ));
                                    vec![cx.text("Blocked")]
                                },
                            );
                            ui.add(element);
                        },
                    );
                });
            })
        },
    );

    let at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-test.float_window.inputs_enabled_false.pressable",
    );
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-no-inputs",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.window_with_options(
                        "demo",
                        "Demo",
                        Point::new(Px(10.0), Px(10.0)),
                        window_behavior_options(fret_ui_kit::imui::FloatingWindowOptions {
                            inputs_enabled: false,
                            ..Default::default()
                        }),
                        |_ui| {},
                    );
                });
            })
        },
    );

    assert!(
        !app.models().get_copied(&clicked_model).unwrap_or(false),
        "expected inputs_enabled=false window to block child pressable activation"
    );
}
