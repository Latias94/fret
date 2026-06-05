use super::*;

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
