use super::*;

#[test]
fn floating_layer_bring_to_front_updates_hit_test_order() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-z-order",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.window("a", "A", Point::new(Px(10.0), Px(10.0)), |ui| {
                        let pressable = ui.cx_mut().pressable(
                            {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(44.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(24.0));
                                props.a11y = fret_ui::element::PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Activate A")),
                                    test_id: Some(Arc::from("imui-test.float_window.activate:a")),
                                    ..Default::default()
                                };
                                props
                            },
                            |cx, _state| vec![cx.text("A")],
                        );
                        ui.add(pressable);
                        let element = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(140.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(80.0));
                                props
                            },
                            |_cx| Vec::new(),
                        );
                        ui.add(element);
                    });
                    ui.window("b", "B", Point::new(Px(60.0), Px(10.0)), |ui| {
                        let pressable = ui.cx_mut().pressable(
                            {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(44.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(24.0));
                                props.a11y = fret_ui::element::PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Activate B")),
                                    test_id: Some(Arc::from("imui-test.float_window.activate:b")),
                                    ..Default::default()
                                };
                                props
                            },
                            |cx, _state| vec![cx.text("B")],
                        );
                        ui.add(pressable);
                        let element = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(140.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(80.0));
                                props
                            },
                            |_cx| Vec::new(),
                        );
                        ui.add(element);
                    });
                });
            })
        },
    );

    let _ = ui.children(root);
    let window_a = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:a",
    );
    let window_b = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:b",
    );

    let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
    let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");
    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap_right = (a_bounds.origin.x.0 + a_bounds.size.width.0)
        .min(b_bounds.origin.x.0 + b_bounds.size.width.0);
    let overlap_bottom = (a_bounds.origin.y.0 + a_bounds.size.height.0)
        .min(b_bounds.origin.y.0 + b_bounds.size.height.0);
    assert!(
        overlap_right > overlap_left + 4.0 && overlap_bottom > overlap_top + 4.0,
        "expected windows to overlap for z-order hit testing"
    );
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let layer_stack = ui.children(root)[0];
    let stack_children = ui.children(layer_stack);
    let stack_idx_a = stack_children
        .iter()
        .position(|n| *n == window_a)
        .expect("expected window A to be a stack child");
    let stack_idx_b = stack_children
        .iter()
        .position(|n| *n == window_b)
        .expect("expected window B to be a stack child");
    assert!(
        stack_idx_b > stack_idx_a,
        "expected window B to be after A initially"
    );

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&window_b),
        "expected window B to be top initially"
    );
    assert!(
        !path.contains(&window_a),
        "expected window A not to be hit initially"
    );

    let activate_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-test.float_window.activate:a",
    );
    click_at(&mut ui, &mut app, &mut services, activate_a);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-z-order",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.window("a", "A", Point::new(Px(10.0), Px(10.0)), |ui| {
                        let pressable = ui.cx_mut().pressable(
                            {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(44.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(24.0));
                                props.a11y = fret_ui::element::PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Activate A")),
                                    test_id: Some(Arc::from("imui-test.float_window.activate:a")),
                                    ..Default::default()
                                };
                                props
                            },
                            |cx, _state| vec![cx.text("A")],
                        );
                        ui.add(pressable);
                        let element = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(140.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(80.0));
                                props
                            },
                            |_cx| Vec::new(),
                        );
                        ui.add(element);
                    });
                    ui.window("b", "B", Point::new(Px(60.0), Px(10.0)), |ui| {
                        let pressable = ui.cx_mut().pressable(
                            {
                                let mut props = fret_ui::element::PressableProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(44.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(24.0));
                                props.a11y = fret_ui::element::PressableA11y {
                                    role: Some(SemanticsRole::Button),
                                    label: Some(Arc::from("Activate B")),
                                    test_id: Some(Arc::from("imui-test.float_window.activate:b")),
                                    ..Default::default()
                                };
                                props
                            },
                            |cx, _state| vec![cx.text("B")],
                        );
                        ui.add(pressable);
                        let element = ui.cx_mut().container(
                            {
                                let mut props = fret_ui::element::ContainerProps::default();
                                props.layout.size.width = fret_ui::element::Length::Px(Px(140.0));
                                props.layout.size.height = fret_ui::element::Length::Px(Px(80.0));
                                props
                            },
                            |_cx| Vec::new(),
                        );
                        ui.add(element);
                    });
                });
            })
        },
    );

    let window_a = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:a",
    );
    let window_b = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:b",
    );

    let layer_stack = ui.children(root)[0];
    let stack_children = ui.children(layer_stack);
    let stack_idx_a = stack_children
        .iter()
        .position(|n| *n == window_a)
        .expect("expected window A to be a stack child");
    let stack_idx_b = stack_children
        .iter()
        .position(|n| *n == window_b)
        .expect("expected window B to be a stack child");
    assert!(
        stack_idx_a > stack_idx_b,
        "expected window A to be after B after activation"
    );

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&window_a),
        "expected window A to be top after activating it"
    );
    assert!(
        !path.contains(&window_b),
        "expected window B not to be hit after activation"
    );
}
