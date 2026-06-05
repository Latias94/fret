use super::*;

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
