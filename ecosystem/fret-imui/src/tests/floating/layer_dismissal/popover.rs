use super::*;

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
