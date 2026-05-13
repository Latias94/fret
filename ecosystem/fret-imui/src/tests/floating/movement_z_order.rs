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
