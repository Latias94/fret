use super::*;

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
