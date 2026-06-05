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
