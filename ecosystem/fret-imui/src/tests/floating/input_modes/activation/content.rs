use super::*;

#[test]
fn floating_window_activate_on_click_can_be_disabled_for_content() {
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
        "imui-floating-layer-activate-on-click-disabled",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions {
                            activate_on_click: false,
                            ..Default::default()
                        }),
                        |ui| {
                            let pressable = ui.cx_mut().pressable(
                                {
                                    let mut props = fret_ui::element::PressableProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(44.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(24.0));
                                    props.a11y = fret_ui::element::PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(Arc::from("Activate A")),
                                        test_id: Some(Arc::from(
                                            "imui-test.float_window.activate_disabled:a",
                                        )),
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
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        },
                    );

                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions::default()),
                        |ui| {
                            let element = ui.cx_mut().container(
                                {
                                    let mut props = fret_ui::element::ContainerProps::default();
                                    props.layout.size.width =
                                        fret_ui::element::Length::Px(Px(140.0));
                                    props.layout.size.height =
                                        fret_ui::element::Length::Px(Px(80.0));
                                    props
                                },
                                |_cx| Vec::new(),
                            );
                            ui.add(element);
                        },
                    );
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
        stack_idx_b > stack_idx_a,
        "expected window B to be after A initially"
    );

    let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
    let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");
    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let hit_before = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path_before = ui.debug_node_path(hit_before);
    assert!(
        path_before.contains(&window_b),
        "expected window B to be top initially"
    );

    let activate_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-test.float_window.activate_disabled:a",
    );
    click_at(&mut ui, &mut app, &mut services, activate_a);

    app.advance_frame();
    let root2 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-activate-on-click-disabled",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions {
                            activate_on_click: false,
                            ..Default::default()
                        }),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions::default()),
                        |_ui| {},
                    );
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

    let layer_stack = ui.children(root2)[0];
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
        "expected window B to remain after A when content activation is disabled"
    );

    let hit_after = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path_after = ui.debug_node_path(hit_after);
    assert!(
        path_after.contains(&window_b),
        "expected window B to remain top after clicking A content when activation is disabled"
    );

    let title_bar_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:a",
    );
    click_at(&mut ui, &mut app, &mut services, title_bar_a);

    app.advance_frame();
    let root3 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-activate-on-click-disabled",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions {
                            activate_on_click: false,
                            ..Default::default()
                        }),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        window_behavior_options(FloatingWindowOptions::default()),
                        |_ui| {},
                    );
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

    let layer_stack = ui.children(root3)[0];
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
        "expected window B to remain after A when activation is disabled"
    );
}
