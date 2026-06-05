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
