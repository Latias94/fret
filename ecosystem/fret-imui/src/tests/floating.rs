use super::*;

#[test]
fn floating_window_moves_when_dragging_title_bar() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-drag",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window("demo", "Demo", Point::new(Px(10.0), Px(10.0)), |ui| {
                    ui.text("Hello");
                });
            })
        },
    );

    let (window_node, _title_bar_node) = floating_window_nodes(&ui, root);
    let before = ui.debug_node_bounds(window_node).expect("window bounds");
    let start = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );

    pointer_down_at(&mut ui, &mut app, &mut services, start);
    let moved = Point::new(Px(start.x.0 + 6.0), start.y);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-drag",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window("demo", "Demo", Point::new(Px(10.0), Px(10.0)), |ui| {
                    ui.text("Hello");
                });
            })
        },
    );

    let (window_node, _title_bar_node) = floating_window_nodes(&ui, root);
    let after = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        after.origin.x.0 > before.origin.x.0,
        "expected floating window to move right"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
}

#[test]
fn floating_area_moves_when_dragging_drag_surface() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-area-drag",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_area("demo", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                    let mut props = fret_ui::element::PointerRegionProps::default();
                    props.layout.size.width = Length::Px(Px(140.0));
                    props.layout.size.height = Length::Px(Px(24.0));
                    let drag = ui
                        .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                        .attach_semantics(
                            fret_ui::element::SemanticsDecoration::default()
                                .test_id(Arc::from("imui.float_area.drag:demo")),
                        );
                    ui.add(drag);
                });
            })
        },
    );

    let area_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:demo",
    );
    let before = ui.debug_node_bounds(area_node).expect("area bounds");
    let start = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.drag:demo",
    );

    pointer_down_at(&mut ui, &mut app, &mut services, start);
    let moved = Point::new(Px(start.x.0 + 6.0), start.y);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-area-drag",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_area("demo", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                    let mut props = fret_ui::element::PointerRegionProps::default();
                    props.layout.size.width = Length::Px(Px(140.0));
                    props.layout.size.height = Length::Px(Px(24.0));
                    let drag = ui
                        .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                        .attach_semantics(
                            fret_ui::element::SemanticsDecoration::default()
                                .test_id(Arc::from("imui.float_area.drag:demo")),
                        );
                    ui.add(drag);
                });
            })
        },
    );

    let area_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:demo",
    );
    let after = ui.debug_node_bounds(area_node).expect("area bounds");
    assert!(
        after.origin.x.0 > before.origin.x.0,
        "expected floating area to move right"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
    let _ = ui.children(root);
}

#[test]
fn floating_area_bring_to_front_updates_hit_test_order() {
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
        "imui-floating-area-z-order",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.floating_area("a", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(80.0));
                        let drag = ui
                            .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:a")),
                            );
                        ui.add(drag);
                    });
                    ui.floating_area("b", Point::new(Px(60.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(80.0));
                        let drag = ui
                            .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:b")),
                            );
                        ui.add(drag);
                    });
                });
            })
        },
    );

    let _ = ui.children(root);
    let area_a = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:a",
    );
    let area_b = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:b",
    );

    let a_bounds = ui.debug_node_bounds(area_a).expect("area a bounds");
    let b_bounds = ui.debug_node_bounds(area_b).expect("area b bounds");

    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap_right = (a_bounds.origin.x.0 + a_bounds.size.width.0)
        .min(b_bounds.origin.x.0 + b_bounds.size.width.0);
    let overlap_bottom = (a_bounds.origin.y.0 + a_bounds.size.height.0)
        .min(b_bounds.origin.y.0 + b_bounds.size.height.0);
    assert!(
        overlap_right > overlap_left + 4.0 && overlap_bottom > overlap_top + 4.0,
        "expected areas to overlap for z-order hit testing"
    );
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let layer_stack = ui.children(root)[0];
    let stack_children = ui.children(layer_stack);
    let stack_idx_a = stack_children
        .iter()
        .position(|n| *n == area_a)
        .expect("expected area A to be a stack child");
    let stack_idx_b = stack_children
        .iter()
        .position(|n| *n == area_b)
        .expect("expected area B to be a stack child");
    assert!(
        stack_idx_b > stack_idx_a,
        "expected area B to be after A initially"
    );

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&area_b),
        "expected area B to be top initially"
    );
    assert!(
        !path.contains(&area_a),
        "expected area A not to be hit initially"
    );

    let handle_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.drag:a",
    );
    click_at(&mut ui, &mut app, &mut services, handle_a);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-area-z-order",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    ui.floating_area("a", Point::new(Px(10.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(80.0));
                        let drag = ui
                            .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:a")),
                            );
                        ui.add(drag);
                    });
                    ui.floating_area("b", Point::new(Px(60.0), Px(10.0)), |ui, area| {
                        let mut props = fret_ui::element::PointerRegionProps::default();
                        props.layout.size.width = Length::Px(Px(140.0));
                        props.layout.size.height = Length::Px(Px(80.0));
                        let drag = ui
                            .floating_area_drag_surface(area, props, |_cx, _id| {}, |_ui| {})
                            .attach_semantics(
                                fret_ui::element::SemanticsDecoration::default()
                                    .test_id(Arc::from("imui.float_area.drag:b")),
                            );
                        ui.add(drag);
                    });
                });
            })
        },
    );

    let area_a = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:a",
    );
    let area_b = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_area.area:b",
    );

    let layer_stack = ui.children(root)[0];
    let stack_children = ui.children(layer_stack);
    let stack_idx_a = stack_children
        .iter()
        .position(|n| *n == area_a)
        .expect("expected area A to be a stack child");
    let stack_idx_b = stack_children
        .iter()
        .position(|n| *n == area_b)
        .expect("expected area B to be a stack child");
    assert!(
        stack_idx_a > stack_idx_b,
        "expected area A to be after B after activation"
    );

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&area_a),
        "expected area A to be top after activating it"
    );
    assert!(
        !path.contains(&area_b),
        "expected area B not to be hit after activation"
    );
}

#[test]
fn window_wrapper_reports_position_and_size() {
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

    let reported_pos = Rc::new(Cell::new(Point::new(Px(0.0), Px(0.0))));
    let reported_size = Rc::new(Cell::new(None::<Size>));

    let reported_pos_out = reported_pos.clone();
    let reported_size_out = reported_size.clone();

    let initial_position = Point::new(Px(10.0), Px(10.0));
    let initial_size = Size::new(Px(140.0), Px(80.0));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-window-wrapper-reports-position-and-size",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    initial_position,
                    resizable_window_options(initial_size),
                    |ui| ui.text("Hello"),
                );
                reported_pos_out.set(resp.position());
                reported_size_out.set(resp.size());
            })
        },
    );

    assert_eq!(reported_pos.get(), initial_position);
    assert_eq!(reported_size.get(), Some(initial_size));
}

#[test]
fn floating_window_close_button_sets_open_false() {
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

    let open = app.models_mut().insert(true);

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options(&open),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let _ = floating_window_nodes(&ui, root);
    let close = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.close:demo",
    );
    click_at(&mut ui, &mut app, &mut services, close);
    assert!(!app.models().get_copied(&open).unwrap_or(true));
}

#[test]
fn floating_window_escape_sets_open_false_after_focusing_title_bar() {
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

    let open = app.models_mut().insert(true);

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-escape",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options(&open),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let _ = floating_window_nodes(&ui, root);
    let title_bar_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    let title_bar_bounds = ui
        .debug_node_bounds(title_bar_node)
        .expect("title bar bounds");
    let title_bar = Point::new(
        Px(title_bar_bounds.origin.x.0 + title_bar_bounds.size.width.0 * 0.5),
        Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 * 0.5),
    );
    click_at(&mut ui, &mut app, &mut services, title_bar);
    assert!(ui.focus().is_some(), "expected title bar to take focus");

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );
    assert!(!app.models().get_copied(&open).unwrap_or(true));
}

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

#[test]
fn floating_window_no_inputs_is_skipped_by_focus_traversal() {
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

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-no-inputs-focus-traversal",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Underlay A",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-a")),
                        ..Default::default()
                    },
                );

                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "overlay",
                        "Overlay",
                        Point::new(Px(120.0), Px(80.0)),
                        window_behavior_options(FloatingWindowOptions {
                            no_inputs: true,
                            ..Default::default()
                        }),
                        |ui| {
                            ui.menu_item_with_options(
                                "Overlay",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-overlay-item")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                });

                ui.menu_item_with_options(
                    "Underlay B",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-b")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let underlay_a_node =
        node_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-underlay-a");
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );
    let _ = ui.dispatch_command(
        &mut app,
        &mut services,
        &fret_runtime::CommandId::from("focus.next"),
    );
    assert_eq!(
        ui.focus(),
        Some(underlay_a_node),
        "expected focus traversal to start at underlay A"
    );

    let _ = ui.dispatch_command(
        &mut app,
        &mut services,
        &fret_runtime::CommandId::from("focus.next"),
    );

    let underlay_b_node =
        node_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-underlay-b");
    let overlay_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-overlay-item",
    );
    assert_eq!(
        ui.focus(),
        Some(underlay_b_node),
        "expected focus traversal to skip no-inputs overlay window"
    );
    assert_ne!(
        ui.focus(),
        Some(overlay_node),
        "expected no-inputs overlay item to be skipped by focus traversal"
    );
}

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

#[test]
fn floating_window_focus_on_click_can_be_independent_from_z_order_activation() {
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

    let fixed = Size::new(Px(200.0), Px(120.0));
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-focus-without-activate",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            fixed,
                            FloatingWindowOptions {
                                activate_on_click: false,
                                focus_on_click: true,
                                ..Default::default()
                            },
                        ),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        resizable_window_options(fixed),
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
    assert!(path_before.contains(&window_b));

    // Click a background point inside window A's content area but outside the overlap area.
    let title_bar_a = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:a",
    );
    let title_bar_bounds = ui
        .debug_node_bounds(title_bar_a)
        .expect("title bar a bounds");
    let click = Point::new(
        Px(a_bounds.origin.x.0 + 30.0),
        Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 + 8.0),
    );
    let hit_click = ui
        .debug_hit_test(click)
        .hit
        .expect("expected click point to hit a node");
    let path_click = ui.debug_node_path(hit_click);
    assert!(
        path_click.contains(&window_a),
        "expected click point to be within window A"
    );
    pointer_down_at(&mut ui, &mut app, &mut services, click);

    let focus = ui
        .focus()
        .expect("expected focus after pointer down on window a");
    let focus_path = ui.debug_node_path(focus);
    assert!(
        focus_path.contains(&window_a),
        "expected focus to be within window A after clicking its background"
    );
    pointer_up_at(&mut ui, &mut app, &mut services, click);

    app.advance_frame();
    let root2 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-focus-without-activate",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            fixed,
                            FloatingWindowOptions {
                                activate_on_click: false,
                                focus_on_click: true,
                                ..Default::default()
                            },
                        ),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        resizable_window_options(fixed),
                        |_ui| {},
                    );
                });
            })
        },
    );

    let window_a2 = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:a",
    );
    let window_b2 = node_for_test_id(
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
        .position(|n| *n == window_a2)
        .expect("expected window A to be a stack child");
    let stack_idx_b = stack_children
        .iter()
        .position(|n| *n == window_b2)
        .expect("expected window B to be a stack child");
    assert!(
        stack_idx_b > stack_idx_a,
        "expected window B to remain after A when activation is disabled"
    );

    let hit_after = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path_after = ui.debug_node_path(hit_after);
    assert!(
        path_after.contains(&window_b2),
        "expected window B to remain top after clicking A background when activation is disabled"
    );
}

#[test]
fn floating_window_activate_on_click_can_be_disabled_for_resize_handles() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(240.0)),
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
        "imui-floating-layer-activate-on-click-disabled-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            Size::new(Px(180.0), Px(120.0)),
                            FloatingWindowOptions {
                                activate_on_click: false,
                                ..Default::default()
                            },
                        ),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(260.0), Px(10.0)),
                        resizable_window_options(Size::new(Px(180.0), Px(120.0))),
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

    let resize_corner_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.resize.corner:a",
    );
    click_at(&mut ui, &mut app, &mut services, resize_corner_a);

    app.advance_frame();
    let root2 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-activate-on-click-disabled-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            Size::new(Px(180.0), Px(120.0)),
                            FloatingWindowOptions {
                                activate_on_click: false,
                                ..Default::default()
                            },
                        ),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(260.0), Px(10.0)),
                        resizable_window_options(Size::new(Px(180.0), Px(120.0))),
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
        "expected window B to remain after A when activation is disabled for resize handles"
    );
}

#[test]
fn floating_window_pointer_passthrough_allows_underlay_hit_testing() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-pointer-passthrough",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let fixed = Size::new(Px(200.0), Px(120.0));
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options(fixed),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            fixed,
                            FloatingWindowOptions {
                                pointer_passthrough: true,
                                ..Default::default()
                            },
                        ),
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
        "expected window B to be after A (painted on top)"
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
        overlap_right > overlap_left && overlap_bottom > overlap_top,
        "expected floating windows to overlap for hit-test passthrough"
    );
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&window_a),
        "expected underlay window A to receive hits through a pass-through window"
    );
    assert!(
        !path.contains(&window_b),
        "expected pass-through window B to be skipped by hit testing"
    );
}

#[test]
fn floating_window_no_inputs_allows_underlay_hit_testing() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-no-inputs-hit-test",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.floating_layer("layer", |ui| {
                    let fixed = Size::new(Px(200.0), Px(120.0));
                    let _ = ui.window_with_options(
                        "a",
                        "A",
                        Point::new(Px(10.0), Px(10.0)),
                        resizable_window_options(fixed),
                        |_ui| {},
                    );
                    let _ = ui.window_with_options(
                        "b",
                        "B",
                        Point::new(Px(60.0), Px(10.0)),
                        resizable_window_options_with_behavior(
                            fixed,
                            FloatingWindowOptions {
                                no_inputs: true,
                                ..Default::default()
                            },
                        ),
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

    let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
    let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");
    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap_right = (a_bounds.origin.x.0 + a_bounds.size.width.0)
        .min(b_bounds.origin.x.0 + b_bounds.size.width.0);
    let overlap_bottom = (a_bounds.origin.y.0 + a_bounds.size.height.0)
        .min(b_bounds.origin.y.0 + b_bounds.size.height.0);
    assert!(
        overlap_right > overlap_left && overlap_bottom > overlap_top,
        "expected floating windows to overlap for no-inputs hit testing"
    );
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let hit = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap point to hit a node");
    let path = ui.debug_node_path(hit);
    assert!(
        path.contains(&window_a),
        "expected underlay window A to receive hits through a no-inputs window"
    );
    assert!(
        !path.contains(&window_b),
        "expected no-inputs window B to be skipped by hit testing"
    );

    // Keep `root` alive to ensure the layer stack is present for debugging.
    let _ = root;
}

#[test]
fn hit_test_passthrough_keeps_focus_traversal_and_nav_highlight() {
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

    let overlay_nav_highlighted = Rc::new(Cell::new(false));
    let overlay_hovered_like_imgui = Rc::new(Cell::new(false));
    let overlay_hovered_no_nav_override = Rc::new(Cell::new(false));

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hit-test-passthrough-focus-traversal",
        |cx| {
            let overlay_nav_highlighted = overlay_nav_highlighted.clone();
            let overlay_hovered_like_imgui = overlay_hovered_like_imgui.clone();
            let overlay_hovered_no_nav_override = overlay_hovered_no_nav_override.clone();
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Underlay",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-item")),
                        ..Default::default()
                    },
                );

                ui.floating_area_with_options(
                    "area",
                    Point::new(Px(0.0), Px(0.0)),
                    FloatingAreaOptions {
                        hit_test_passthrough: true,
                        ..Default::default()
                    },
                    |ui, _area| {
                        let resp = ui.menu_item_with_options(
                            "Overlay",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-overlay-item")),
                                ..Default::default()
                            },
                        );
                        overlay_nav_highlighted.set(resp.nav_highlighted());
                        overlay_hovered_like_imgui.set(resp.hovered_like_imgui());
                        overlay_hovered_no_nav_override
                            .set(resp.is_hovered(ImUiHoveredFlags::NO_NAV_OVERRIDE));
                    },
                );
            })
        },
    );

    // Clicking the overlay item should focus the underlay item because the overlay subtree is
    // hit-test transparent.
    let overlay_bounds = bounds_for_test_id(&ui, "imui-overlay-item");
    let overlay_center = Point::new(
        Px(overlay_bounds.origin.x.0 + overlay_bounds.size.width.0 * 0.5),
        Px(overlay_bounds.origin.y.0 + overlay_bounds.size.height.0 * 0.5),
    );
    click_at(&mut ui, &mut app, &mut services, overlay_center);

    let underlay_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-underlay-item",
    );
    assert_eq!(
        ui.focus(),
        Some(underlay_node),
        "expected click-through to focus the underlay item"
    );

    // Simulate keyboard navigation becoming active (focus-visible), then traverse to the next
    // focusable item. The overlay subtree should still participate in focus traversal even
    // though it is pointer-transparent.
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );
    let _ = ui.dispatch_command(
        &mut app,
        &mut services,
        &fret_runtime::CommandId::from("focus.next"),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _ = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-hit-test-passthrough-focus-traversal",
        |cx| {
            let overlay_nav_highlighted = overlay_nav_highlighted.clone();
            let overlay_hovered_like_imgui = overlay_hovered_like_imgui.clone();
            let overlay_hovered_no_nav_override = overlay_hovered_no_nav_override.clone();
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Underlay",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-item")),
                        ..Default::default()
                    },
                );
                ui.floating_area_with_options(
                    "area",
                    Point::new(Px(0.0), Px(0.0)),
                    FloatingAreaOptions {
                        hit_test_passthrough: true,
                        ..Default::default()
                    },
                    |ui, _area| {
                        let resp = ui.menu_item_with_options(
                            "Overlay",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-overlay-item")),
                                ..Default::default()
                            },
                        );
                        overlay_nav_highlighted.set(resp.nav_highlighted());
                        overlay_hovered_like_imgui.set(resp.hovered_like_imgui());
                        overlay_hovered_no_nav_override
                            .set(resp.is_hovered(ImUiHoveredFlags::NO_NAV_OVERRIDE));
                    },
                );
            })
        },
    );

    let overlay_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-overlay-item",
    );
    assert_eq!(
        ui.focus(),
        Some(overlay_node),
        "expected focus traversal to reach pointer-transparent overlay item"
    );
    assert!(
        overlay_nav_highlighted.get(),
        "expected overlay item to report nav highlight when focus-visible is active"
    );
    assert!(
        overlay_hovered_like_imgui.get(),
        "expected hovered_like_imgui to be true under nav highlight"
    );
    assert!(
        !overlay_hovered_no_nav_override.get(),
        "expected NoNavOverride hovered query to ignore nav highlight"
    );
}

#[test]
fn no_inputs_is_click_through_and_skips_focus_traversal() {
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

    let overlay_nav_highlighted = Rc::new(Cell::new(false));
    let overlay_hovered_like_imgui = Rc::new(Cell::new(false));

    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-no-inputs-focus-traversal",
        |cx| {
            let overlay_nav_highlighted = overlay_nav_highlighted.clone();
            let overlay_hovered_like_imgui = overlay_hovered_like_imgui.clone();
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Underlay A",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-a")),
                        ..Default::default()
                    },
                );

                ui.floating_area_with_options(
                    "area",
                    Point::new(Px(0.0), Px(0.0)),
                    FloatingAreaOptions {
                        hit_test_passthrough: true,
                        no_inputs: true,
                        ..Default::default()
                    },
                    |ui, _area| {
                        let resp = ui.menu_item_with_options(
                            "Overlay",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-overlay-item")),
                                ..Default::default()
                            },
                        );
                        overlay_nav_highlighted.set(resp.nav_highlighted());
                        overlay_hovered_like_imgui.set(resp.hovered_like_imgui());
                    },
                );

                ui.menu_item_with_options(
                    "Underlay B",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-b")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let underlay_a_node =
        node_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-underlay-a");
    let underlay_b_node =
        node_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-underlay-b");
    let overlay_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-overlay-item",
    );

    // The overlay subtree should be hit-test transparent.
    let overlay_bounds = bounds_for_test_id(&ui, "imui-overlay-item");
    let overlay_center = Point::new(
        Px(overlay_bounds.origin.x.0 + overlay_bounds.size.width.0 * 0.5),
        Px(overlay_bounds.origin.y.0 + overlay_bounds.size.height.0 * 0.5),
    );
    let hit = ui
        .debug_hit_test(overlay_center)
        .hit
        .expect("expected overlay point to hit an underlay node");
    let path = ui.debug_node_path(hit);
    assert!(
        !path.contains(&overlay_node),
        "expected no-inputs overlay subtree to be skipped by hit testing"
    );
    assert!(
        path.contains(&underlay_a_node) || path.contains(&underlay_b_node),
        "expected an underlay node to receive hits under the overlay point"
    );

    // Clicking the overlay should not focus the overlay subtree.
    click_at(&mut ui, &mut app, &mut services, overlay_center);
    assert_ne!(
        ui.focus(),
        Some(overlay_node),
        "expected click-through not to focus the overlay subtree"
    );

    // Focus traversal should skip the overlay subtree entirely.
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );
    for _ in 0..4 {
        let _ = ui.dispatch_command(
            &mut app,
            &mut services,
            &fret_runtime::CommandId::from("focus.next"),
        );
        assert_ne!(ui.focus(), Some(overlay_node));
    }

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _ = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-no-inputs-focus-traversal",
        |cx| {
            let overlay_nav_highlighted = overlay_nav_highlighted.clone();
            let overlay_hovered_like_imgui = overlay_hovered_like_imgui.clone();
            crate::imui_raw(cx, |ui| {
                ui.menu_item_with_options(
                    "Underlay A",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-a")),
                        ..Default::default()
                    },
                );
                ui.floating_area_with_options(
                    "area",
                    Point::new(Px(0.0), Px(0.0)),
                    FloatingAreaOptions {
                        hit_test_passthrough: true,
                        no_inputs: true,
                        ..Default::default()
                    },
                    |ui, _area| {
                        let resp = ui.menu_item_with_options(
                            "Overlay",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-overlay-item")),
                                ..Default::default()
                            },
                        );
                        overlay_nav_highlighted.set(resp.nav_highlighted());
                        overlay_hovered_like_imgui.set(resp.hovered_like_imgui());
                    },
                );
                ui.menu_item_with_options(
                    "Underlay B",
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-underlay-b")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let overlay_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-overlay-item",
    );
    assert_ne!(ui.focus(), Some(overlay_node));
    assert!(
        !overlay_nav_highlighted.get(),
        "expected overlay item not to report nav highlight when no_inputs is enabled"
    );
    assert!(
        !overlay_hovered_like_imgui.get(),
        "expected hovered_like_imgui to be false when no_inputs is enabled"
    );
}

#[test]
fn floating_layer_menu_outside_press_dismisses_without_activating_underlay() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(280.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let open = app.models_mut().insert(false);
    let overlay_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-menu-dismiss-no-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Menu,
                overlay_id_out.clone(),
            )
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

    let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
    let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");
    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let hit_before = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_before = ui.debug_node_path(hit_before);
    assert!(
        path_before.contains(&window_b),
        "expected window B to be top initially"
    );

    // Bring A to the front before opening the overlay so we can observe whether B activates
    // as a result of an outside press.
    let title_bar_a = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:a",
    );
    click_at(&mut ui, &mut app, &mut services, title_bar_a);

    app.advance_frame();
    let _root2 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-menu-dismiss-no-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Menu,
                overlay_id_out.clone(),
            )
        },
    );

    let hit_open = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_open = ui.debug_node_path(hit_open);
    assert!(
        path_open.contains(&window_a),
        "expected window A to be top after activation"
    );

    // Open the overlay programmatically to avoid relying on hit-testable trigger semantics.
    app.models_mut()
        .update(&open, |v| *v = true)
        .expect("open model update");
    app.advance_frame();
    let _root3 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-menu-dismiss-no-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Menu,
                overlay_id_out.clone(),
            )
        },
    );

    let overlay_id = overlay_id_out.get().expect("overlay id should be captured");
    let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert_eq!(
        snap.topmost_popover,
        Some(overlay_id),
        "expected menu overlay to be the topmost popover"
    );
    assert_ne!(
        snap.arbitration.pointer_occlusion,
        PointerOcclusion::None,
        "expected menu overlay to enable pointer occlusion (disableOutsidePointerEvents)"
    );

    let click_b = Point::new(
        Px(b_bounds.origin.x.0 + b_bounds.size.width.0 - 6.0),
        Px(b_bounds.origin.y.0 + 6.0),
    );
    click_at(&mut ui, &mut app, &mut services, click_b);

    app.advance_frame();
    let _root3 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-menu-dismiss-no-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Menu,
                overlay_id_out.clone(),
            )
        },
    );

    assert!(
        !app.models().get_copied(&open).unwrap_or(true),
        "expected outside press to dismiss the menu"
    );
    let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert_eq!(
        snap.topmost_popover, None,
        "expected menu overlay to be dismissed"
    );

    let hit_after = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_after = ui.debug_node_path(hit_after);
    assert!(
        path_after.contains(&window_a),
        "expected window A to remain top after non-click-through outside press"
    );
    assert!(
        !path_after.contains(&window_b),
        "expected window B not to activate on a non-click-through outside press"
    );
}

#[test]
fn floating_layer_popover_outside_press_allows_underlay_activation_when_click_through() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(480.0), Px(280.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let open = app.models_mut().insert(false);
    let overlay_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-popover-dismiss-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Popover,
                overlay_id_out.clone(),
            )
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

    let a_bounds = ui.debug_node_bounds(window_a).expect("window a bounds");
    let b_bounds = ui.debug_node_bounds(window_b).expect("window b bounds");
    let overlap_left = a_bounds.origin.x.0.max(b_bounds.origin.x.0);
    let overlap_top = a_bounds.origin.y.0.max(b_bounds.origin.y.0);
    let overlap = Point::new(Px(overlap_left + 2.0), Px(overlap_top + 2.0));

    let hit_before = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_before = ui.debug_node_path(hit_before);
    assert!(
        path_before.contains(&window_b),
        "expected window B to be top initially"
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
    let _root2 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-popover-dismiss-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Popover,
                overlay_id_out.clone(),
            )
        },
    );

    let hit_open = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_open = ui.debug_node_path(hit_open);
    assert!(
        path_open.contains(&window_a),
        "expected window A to be top after activation"
    );

    app.models_mut()
        .update(&open, |v| *v = true)
        .expect("open model update");
    app.advance_frame();
    let _root3 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-popover-dismiss-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Popover,
                overlay_id_out.clone(),
            )
        },
    );

    let overlay_id = overlay_id_out.get().expect("overlay id should be captured");
    let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert_eq!(
        snap.topmost_popover,
        Some(overlay_id),
        "expected popover to be the topmost popover"
    );
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        PointerOcclusion::None,
        "expected click-through popover to not enable pointer occlusion"
    );
    let popover_entry = snap
        .stack
        .iter()
        .rev()
        .find(|e| e.id == Some(overlay_id))
        .expect("expected popover stack entry");
    assert!(
        !popover_entry.blocks_underlay_input,
        "expected click-through popover to not block underlay input"
    );

    let window_b_now = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:b",
    );
    let b_bounds_now = ui.debug_node_bounds(window_b_now).expect("window b bounds");
    let click_b = Point::new(
        Px(b_bounds_now.origin.x.0 + b_bounds_now.size.width.0 - 6.0),
        Px(b_bounds_now.origin.y.0 + 40.0),
    );
    let hit_click = ui
        .debug_hit_test(click_b)
        .hit
        .expect("expected click point to hit a node");
    let path_click = ui.debug_node_path(hit_click);
    assert!(
        path_click.contains(&window_b_now),
        "expected click point to hit window B"
    );
    click_at(&mut ui, &mut app, &mut services, click_b);

    app.advance_frame();
    let _root3 = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-layer-popover-dismiss-click-through",
        |cx| {
            render_floating_layer_with_overlay(
                cx,
                open.clone(),
                FloatingLayerOverlayVariant::Popover,
                overlay_id_out.clone(),
            )
        },
    );

    assert!(
        !app.models().get_copied(&open).unwrap_or(true),
        "expected outside press to dismiss the popover"
    );
    let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert_eq!(
        snap.topmost_popover, None,
        "expected popover to be dismissed"
    );

    let hit_after = ui
        .debug_hit_test(overlap)
        .hit
        .expect("expected overlap to hit");
    let path_after = ui.debug_node_path(hit_after);
    assert!(
        path_after.contains(&window_b),
        "expected window B to activate on click-through outside press"
    );
}

#[test]
fn floating_window_closable_false_hides_close_button_and_escape_does_not_close() {
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

    let open = app.models_mut().insert(true);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.close:demo",
        ),
        "expected close button to be hidden when closable=false"
    );

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    click_at(&mut ui, &mut app, &mut services, title_bar);
    assert!(ui.focus().is_some(), "expected title bar to take focus");

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-closable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    open_window_options_with_behavior(
                        &open,
                        FloatingWindowOptions {
                            closable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        app.models().get_copied(&open).unwrap_or(false),
        "expected Escape not to close when closable=false"
    );
}

#[test]
fn floating_window_movable_false_does_not_move_when_dragging_title_bar() {
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

    let position = Rc::new(Cell::new(Point::default()));

    let position_out = position.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-movable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    window_behavior_options(FloatingWindowOptions {
                        movable: false,
                        ..Default::default()
                    }),
                    |ui| ui.text("Hello"),
                );
                position_out.set(resp.position());
            })
        },
    );
    let _ = ui.children(root);
    let before = position.get();

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, title_bar);
    let moved = Point::new(Px(title_bar.x.0 + 30.0), Px(title_bar.y.0 + 8.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );
    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);

    app.advance_frame();
    let position_out = position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-movable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    window_behavior_options(FloatingWindowOptions {
                        movable: false,
                        ..Default::default()
                    }),
                    |ui| ui.text("Hello"),
                );
                position_out.set(resp.position());
            })
        },
    );

    assert_eq!(
        position.get(),
        before,
        "expected window position unchanged when movable=false"
    );
}

#[test]
fn floating_window_resizable_false_hides_resize_handles() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resizable-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            resizable: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles hidden when resizable=false"
    );
}

#[test]
fn floating_window_collapsible_false_does_not_toggle_on_title_double_click() {
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

    let collapsed = Rc::new(Cell::new(false));

    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );
    assert!(!collapsed.get());

    let title_bar = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapsible-false",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options_with_behavior(
                        Size::new(Px(180.0), Px(120.0)),
                        FloatingWindowOptions {
                            collapsible: false,
                            ..Default::default()
                        },
                    ),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
            })
        },
    );

    assert!(
        !collapsed.get(),
        "expected title-bar double click not to toggle collapse when collapsible=false"
    );
}

#[test]
fn floating_window_resizes_when_dragging_corner_handle() {
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

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let corner = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.resize.corner:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, corner);
    let moved = Point::new(Px(corner.x.0 + 20.0), Px(corner.y.0 + 10.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(10.0), Px(10.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| {
                        ui.text("Hello");
                    },
                );
            })
        },
    );
    let _ = ui.children(root);

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let after = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        after.size.width.0 > before.size.width.0,
        "expected window to grow wider"
    );
    assert!(
        after.size.height.0 > before.size.height.0,
        "expected window to grow taller"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
}

#[test]
fn floating_window_resizes_from_left_updates_origin_and_width() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize-left",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(80.0), Px(40.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );

    let _ = ui.children(root);
    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let left = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.resize.left:demo",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, left);
    let moved = Point::new(Px(left.x.0 - 18.0), left.y);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        moved,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-resize-left",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(80.0), Px(40.0)),
                    resizable_window_options(Size::new(Px(140.0), Px(80.0))),
                    |ui| ui.text("Hello"),
                );
            })
        },
    );
    let _ = ui.children(root);

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let after = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        after.origin.x.0 < before.origin.x.0,
        "expected origin.x to move left when resizing from left"
    );
    assert!(
        after.size.width.0 > before.size.width.0,
        "expected width to grow when resizing from left"
    );

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, moved, false);
}

#[test]
fn floating_window_title_bar_double_click_toggles_collapsed() {
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

    let collapsed = Rc::new(Cell::new(false));
    let resizing = Rc::new(Cell::new(false));
    let area_id = Rc::new(Cell::new(0u64));

    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.area.id.0);
            })
        },
    );
    let _ = ui.children(root);
    assert!(!collapsed.get());
    assert!(!resizing.get());
    let area_id_before = area_id.get();
    assert_ne!(area_id_before, 0, "expected non-zero floating area id");

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let before = ui.debug_node_bounds(window_node).expect("window bounds");

    let title_bar_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    let title_bar_bounds = ui
        .debug_node_bounds(title_bar_node)
        .expect("title bar bounds");
    let title_bar = Point::new(
        Px(title_bar_bounds.origin.x.0 + title_bar_bounds.size.width.0 * 0.5),
        Px(title_bar_bounds.origin.y.0 + title_bar_bounds.size.height.0 * 0.5),
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.area.id.0);
            })
        },
    );
    assert!(collapsed.get());
    assert!(!resizing.get());
    let area_id_collapsed = area_id.get();
    assert_eq!(
        area_id_collapsed, area_id_before,
        "expected floating area id stable across collapse"
    );

    let window_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.window:demo",
    );
    let collapsed_bounds = ui.debug_node_bounds(window_node).expect("window bounds");
    assert!(
        collapsed_bounds.size.height.0 < before.size.height.0,
        "expected collapsed window to be shorter"
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles hidden while collapsed"
    );

    let title_bar_after_collapse_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui.float_window.title_bar:demo",
    );
    let title_bar_after_collapse_bounds = ui
        .debug_node_bounds(title_bar_after_collapse_node)
        .expect("title bar bounds");
    let title_bar_after_collapse = Point::new(
        Px(title_bar_after_collapse_bounds.origin.x.0
            + title_bar_after_collapse_bounds.size.width.0 * 0.5),
        Px(title_bar_after_collapse_bounds.origin.y.0
            + title_bar_after_collapse_bounds.size.height.0 * 0.5),
    );
    double_click_at(&mut ui, &mut app, &mut services, title_bar_after_collapse);

    app.advance_frame();
    let collapsed_out = collapsed.clone();
    let resizing_out = resizing.clone();
    let area_id_out = area_id.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-floating-window-collapse",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.window_with_options(
                    "demo",
                    "Demo",
                    Point::new(Px(60.0), Px(36.0)),
                    resizable_window_options(Size::new(Px(180.0), Px(120.0))),
                    |ui| ui.text("Hello"),
                );
                collapsed_out.set(resp.collapsed());
                resizing_out.set(resp.resizing());
                area_id_out.set(resp.area.id.0);
            })
        },
    );
    assert!(!collapsed.get());
    assert!(!resizing.get());
    assert_eq!(
        area_id.get(),
        area_id_before,
        "expected floating area id stable across expand"
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui.float_window.resize.corner:demo",
        ),
        "expected resize handles restored after expanding"
    );
}
