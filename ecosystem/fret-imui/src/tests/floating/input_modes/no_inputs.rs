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
