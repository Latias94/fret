use super::*;

#[test]
fn drag_preview_ghost_follows_pointer_and_clears_on_release() {
    fn subtree_contains_bounds(
        ui: &UiTree<TestHost>,
        node: fret_core::NodeId,
        expected: Rect,
    ) -> bool {
        if ui.debug_node_bounds(node) == Some(expected) {
            return true;
        }

        ui.children(node)
            .iter()
            .copied()
            .any(|child| subtree_contains_bounds(ui, child, expected))
    }

    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let source_position = Rc::new(Cell::new(None::<Point>));
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert_eq!(ui.layer_ids_in_paint_order().len(), 1);

    let source_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert_eq!(source_position.get(), Some(drag_point));
    let ghost_layer = *ui
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer");
    assert!(ui.is_layer_visible(ghost_layer));
    let ghost_root = ui.layer_root(ghost_layer).expect("ghost layer root");
    let expected_bounds = Rect::new(
        Point::new(Px(drag_point.x.0 + 12.0), Px(drag_point.y.0 + 12.0)),
        Size::new(Px(96.0), Px(28.0)),
    );
    assert!(subtree_contains_bounds(&ui, ghost_root, expected_bounds));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, drag_point, false);

    app.advance_frame();
    let source_position_out = source_position.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-preview-ghost",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let source = ui.button_with_options(
                    "Asset",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-drag-preview-source")),
                        ..Default::default()
                    },
                );
                let source_state = ui.drag_source(
                    source,
                    TestDragPayload {
                        label: Arc::from("Stone"),
                    },
                );
                source_position_out.set(source_state.position());

                let _ = drag_preview_ghost_with_options(
                    ui,
                    "asset-ghost",
                    source_state,
                    DragPreviewGhostOptions {
                        test_id: Some(Arc::from("imui-drag-preview-ghost")),
                        ..Default::default()
                    },
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    }),
                );
            })
        },
    );

    assert!(source_position.get().is_none());
    assert!(!ui.is_layer_visible(ghost_layer));
}
#[test]
fn cross_window_drag_preview_ghost_transfers_between_windows() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui_a = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b = UiTree::new();
    ui_b.set_window(window_b);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render_scene = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let source = ui.button_with_options(
                "Asset",
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-source")),
                    ..Default::default()
                },
            );
            let source_state = ui.drag_source(
                source,
                TestDragPayload {
                    label: Arc::from("Stone"),
                },
            );

            let _ = publish_cross_window_drag_preview_ghost_with_options(
                ui,
                "asset-ghost",
                source_state,
                DragPreviewGhostOptions {
                    test_id: Some(Arc::from("imui-cross-window-drag-preview-ghost")),
                    ..Default::default()
                },
                |_cx| {
                    fret_ui_kit::ui::container_build(|cx, out| {
                        let mut props = fret_ui::element::ContainerProps::default();
                        props.layout.size.width = fret_ui::element::Length::Px(Px(96.0));
                        props.layout.size.height = fret_ui::element::Length::Px(Px(28.0));
                        out.push(cx.container(props, |cx| vec![cx.text("Stone")]));
                    })
                },
            );

            let _ = render_cross_window_drag_preview_ghosts(ui.cx_mut());
        })
    };

    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    assert!(!has_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let source_point = point_for_test_id(
        &mut ui_a,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-source",
    );
    let drag_point = Point::new(Px(source_point.x.0 + 24.0), Px(source_point.y.0 + 18.0));

    pointer_down_at(&mut ui_a, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui_a,
        &mut app,
        &mut services,
        drag_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    let ghost_layer_a = *ui_a
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window a");
    assert!(ui_a.is_layer_visible(ghost_layer_a));
    assert!(!has_test_id(
        &mut ui_b,
        &mut app,
        &mut services,
        bounds,
        "imui-cross-window-drag-preview-ghost",
    ));

    let drag = app.drag_mut(PointerId(0)).expect("drag session");
    drag.current_window = window_b;
    drag.cross_window_hover = true;
    drag.position = Point::new(Px(120.0), Px(72.0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    let ghost_layer_b = *ui_b
        .layer_ids_in_paint_order()
        .last()
        .expect("expected ghost overlay layer in window b");
    assert!(ui_b.is_layer_visible(ghost_layer_b));

    app.cancel_drag(PointerId(0));

    app.advance_frame();
    let _ = run_frame(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );
    let _ = run_frame(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        "imui-cross-window-drag-preview",
        render_scene,
    );

    assert!(!ui_a.is_layer_visible(ghost_layer_a));
    assert!(!ui_b.is_layer_visible(ghost_layer_b));
}
