use super::*;

#[test]
fn drag_started_stopped_and_delta_are_consistent() {
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

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let stopped = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move below the threshold.
    let p1 = Point::new(Px(start.x.0 + 2.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(!stopped.get());

    // Move past the threshold to start dragging (delta should be the frame delta, not the total).
    let p2 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert!(!stopped.get());
    assert_eq!(delta.get(), Point::new(Px(4.0), Px(0.0)));

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, p2, false);

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let stopped_out = stopped.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-signals",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                stopped_out.set(resp.drag_stopped());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());
    assert!(stopped.get());
}
#[test]
fn drag_threshold_metric_controls_drag_start() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    fret_ui::Theme::with_global_mut(&mut app, |theme| {
        let mut cfg = fret_ui::theme::ThemeConfig {
            name: "Test".to_string(),
            ..fret_ui::theme::ThemeConfig::default()
        };
        cfg.metrics
            .insert("component.imui.drag_threshold_px".to_string(), 7.0);
        theme.apply_config_patch(&cfg);
    });
    let mut services = FakeTextService::default();

    let started = Rc::new(Cell::new(false));
    let dragging = Rc::new(Cell::new(false));
    let delta = Rc::new(Cell::new(Point::default()));

    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    let start = first_child_point(&ui, root);
    pointer_down_at(&mut ui, &mut app, &mut services, start);

    // Move below the configured threshold (7px).
    let p1 = Point::new(Px(start.x.0 + 6.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p1,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(!started.get());
    assert!(!dragging.get());

    // Move past the threshold; delta should be the frame delta (8 - 6 = 2).
    let p2 = Point::new(Px(start.x.0 + 8.0), Px(start.y.0));
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        p2,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let started_out = started.clone();
    let dragging_out = dragging.clone();
    let delta_out = delta.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-threshold-metric",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                started_out.set(resp.drag_started());
                dragging_out.set(resp.dragging());
                delta_out.set(resp.drag_delta());
            })
        },
    );
    assert!(started.get());
    assert!(dragging.get());
    assert_eq!(delta.get(), Point::new(Px(2.0), Px(0.0)));
}
#[test]
fn drag_drop_helper_previews_and_delivers_payload() {
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

    let source_active = Rc::new(Cell::new(false));
    let target_over = Rc::new(Cell::new(false));
    let delivered = Rc::new(Cell::new(false));
    let preview_label = Rc::new(RefCell::new(String::new()));
    let delivered_label = Rc::new(RefCell::new(String::new()));

    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(!delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert!(delivered_label.borrow().is_empty());

    let source_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-source");
    let target_point =
        point_for_test_id(&mut ui, &mut app, &mut services, bounds, "imui-dnd-target");

    pointer_down_at(&mut ui, &mut app, &mut services, source_point);
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        target_point,
        MouseButtons {
            left: true,
            ..MouseButtons::default()
        },
    );

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(source_active.get());
    assert!(target_over.get());
    assert!(!delivered.get());
    assert_eq!(preview_label.borrow().as_str(), "Stone");
    assert!(delivered_label.borrow().is_empty());

    pointer_up_at_with_is_click(&mut ui, &mut app, &mut services, target_point, false);

    app.advance_frame();
    let source_active_out = source_active.clone();
    let target_over_out = target_over.clone();
    let delivered_out = delivered.clone();
    let preview_label_out = preview_label.clone();
    let delivered_label_out = delivered_label.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-drag-drop-helper",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.horizontal(|ui| {
                    let source = ui.button_with_options(
                        "Asset",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-source")),
                            ..Default::default()
                        },
                    );
                    let source_state = ui.drag_source(
                        source,
                        TestDragPayload {
                            label: Arc::from("Stone"),
                        },
                    );
                    source_active_out.set(source_state.active());

                    let target = ui.button_with_options(
                        "Slot",
                        fret_ui_kit::imui::ButtonOptions {
                            test_id: Some(Arc::from("imui-dnd-target")),
                            ..Default::default()
                        },
                    );
                    let drop = ui.drop_target::<TestDragPayload>(target);
                    target_over_out.set(drop.over());
                    delivered_out.set(drop.delivered());
                    preview_label_out.replace(
                        drop.preview_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                    delivered_label_out.replace(
                        drop.delivered_payload()
                            .map(|payload| payload.label.as_ref().to_string())
                            .unwrap_or_default(),
                    );
                });
            })
        },
    );

    assert!(!source_active.get());
    assert!(!target_over.get());
    assert!(delivered.get());
    assert!(preview_label.borrow().is_empty());
    assert_eq!(delivered_label.borrow().as_str(), "Stone");
}
