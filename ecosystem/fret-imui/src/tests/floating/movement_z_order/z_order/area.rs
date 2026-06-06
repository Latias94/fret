use super::*;

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
