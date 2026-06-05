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
