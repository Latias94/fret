use super::*;

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
