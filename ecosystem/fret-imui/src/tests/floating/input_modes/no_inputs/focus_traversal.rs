use super::*;

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
