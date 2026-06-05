use super::*;

#[test]
fn base_control_state_changes_keep_outer_bounds_stable() {
    use fret_ui_kit::imui::{ButtonOptions, RadioOptions};

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(360.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let checkbox = app.models_mut().insert(false);
    let switch = app.models_mut().insert(false);
    let slider = app.models_mut().insert(0.25f32);
    let radio_selected = Rc::new(Cell::new(false));
    let selectable_selected = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.button_with_options(
                    "Button",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-geometry.button")),
                        ..Default::default()
                    },
                );
                let _ = ui.checkbox_model_with_options(
                    "Checkbox",
                    &checkbox,
                    CheckboxOptions {
                        test_id: Some(Arc::from("imui-geometry.checkbox")),
                        ..Default::default()
                    },
                );
                let _ = ui.radio_with_options(
                    "Radio",
                    radio_selected.get(),
                    RadioOptions {
                        test_id: Some(Arc::from("imui-geometry.radio")),
                        ..Default::default()
                    },
                );
                let _ = ui.switch_model_with_options(
                    "Switch",
                    &switch,
                    SwitchOptions {
                        test_id: Some(Arc::from("imui-geometry.switch")),
                        ..Default::default()
                    },
                );
                let _ = ui.slider_f32_model_with_options(
                    "Slider",
                    &slider,
                    SliderOptions {
                        test_id: Some(Arc::from("imui-geometry.slider")),
                        min: 0.0,
                        max: 1.0,
                        step: 0.01,
                        ..Default::default()
                    },
                );
                let _ = ui.combo_with_options(
                    "geometry-combo",
                    "Mode",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-geometry.combo")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-geometry.combo.alpha")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let _ = ui.selectable_with_options(
                    "Selectable",
                    SelectableOptions {
                        selected: selectable_selected.get(),
                        test_id: Some(Arc::from("imui-geometry.selectable")),
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
        "imui-base-control-geometry",
        |cx| render(cx),
    );

    let control_ids = [
        "imui-geometry.button",
        "imui-geometry.checkbox",
        "imui-geometry.radio",
        "imui-geometry.switch",
        "imui-geometry.slider",
        "imui-geometry.combo",
        "imui-geometry.selectable",
    ];
    let mut baseline = Vec::new();
    for test_id in control_ids {
        baseline.push((
            test_id,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
        ));
    }

    for test_id in control_ids {
        let before = baseline_bounds(&baseline, test_id);
        pointer_move_at(
            &mut ui,
            &mut app,
            &mut services,
            center_of_rect(before),
            MouseButtons::default(),
        );
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-base-control-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "hover",
        );

        let node = node_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id);
        ui.set_focus(Some(node));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-base-control-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "focus",
        );

        pointer_down_at(&mut ui, &mut app, &mut services, center_of_rect(before));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-base-control-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "pressed",
        );
        pointer_up_at_with_is_click(
            &mut ui,
            &mut app,
            &mut services,
            center_of_rect(before),
            false,
        );
    }

    for test_id in [
        "imui-geometry.checkbox",
        "imui-geometry.switch",
        "imui-geometry.slider",
        "imui-geometry.combo",
    ] {
        let before = baseline_bounds(&baseline, test_id);
        click_at(&mut ui, &mut app, &mut services, center_of_rect(before));
        advance_and_run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-base-control-geometry",
            &render,
        );
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "value/open",
        );
    }

    radio_selected.set(true);
    selectable_selected.set(true);
    advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-base-control-geometry",
        &render,
    );
    for test_id in ["imui-geometry.radio", "imui-geometry.selectable"] {
        let before = baseline_bounds(&baseline, test_id);
        assert_same_rect(
            test_id,
            before,
            control_bounds_for_test_id(&mut ui, &mut app, &mut services, bounds, test_id),
            "selected",
        );
    }
}
