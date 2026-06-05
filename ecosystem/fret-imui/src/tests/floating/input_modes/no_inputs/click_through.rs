use super::*;

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
