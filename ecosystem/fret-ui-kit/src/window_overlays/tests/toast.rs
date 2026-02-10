use super::*;

#[test]
fn toast_viewport_focus_command_focuses_active_toast_layer() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    let store = toast_store(&mut app);
    let _ = toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello"),
    );

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone()),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .toast_layers
            .get(&(window, viewport_id))
            .map(|l| l.layer)
            .expect("toast layer installed")
    });
    let expected = ui.layer_root(layer).expect("layer root");

    assert!(try_handle_window_command(
        &mut ui,
        &mut app,
        window,
        &fret_runtime::CommandId::from(TOAST_VIEWPORT_FOCUS_COMMAND),
    ));
    assert_eq!(ui.focus(), Some(expected));
}

#[test]
fn toast_hit_testing_tracks_render_transform() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(600.0), Px(400.0)),
    );

    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_cx| Vec::new(),
    );
    ui.set_root(base);

    let store = toast_store(&mut app);
    let toast_id = toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello").duration(None),
    );

    // Simulate a completed drag to install a non-zero settle offset (render translation) for the
    // first frame.
    let (started, moved, ended) = app
        .models_mut()
        .update(&store, |st| {
            let _ = st.set_window_swipe_config(window, ToastSwipeDirection::Right, Px(1000.0));
            let started = st.begin_drag(window, toast_id, Point::new(Px(0.0), Px(0.0)));
            let moved = st
                .drag_move(window, toast_id, Point::new(Px(200.0), Px(0.0)))
                .is_some();
            let ended = st.end_drag(window, toast_id).is_some();
            (started, moved, ended)
        })
        .expect("toast drag state update");
    assert!(started && moved && ended);

    let (settle_from, drag_offset, dragging) = app
        .models()
        .read(&store, |st| {
            let toast = st
                .toasts_for_window(window)
                .iter()
                .find(|t| t.id == toast_id)
                .expect("toast present");
            (toast.settle_from, toast.drag_offset, toast.dragging)
        })
        .expect("toast state snapshot");
    assert!(settle_from.is_some());
    assert_eq!(drag_offset, Point::new(Px(0.0), Px(0.0)));
    assert!(!dragging);

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone())
            .position(ToastPosition::TopLeft)
            .margin(Px(0.0))
            .gap(Px(0.0)),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let toast_count = app
        .models()
        .read(&store, |st| st.toasts_for_window(window).len())
        .unwrap_or_default();
    assert!(toast_count > 0);

    let toast_layer = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
        overlays
            .toast_layers
            .get(&(window, viewport_id))
            .map(|l| l.layer)
            .expect("toast layer installed")
    });
    let layer_info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == toast_layer)
        .expect("toast layer info");
    assert!(layer_info.visible);
    assert!(layer_info.hit_testable);

    // Find a point that's inside the visually translated toast (settle offset), but outside the
    // untransformed layout bounds. This should still hit the toast and install a drag start.
    let mut hit = false;
    for y in (8..=360).step_by(16) {
        for x in (300..=580).step_by(10) {
            let pos = Point::new(Px(x as f32), Px(y as f32));
            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                    position: pos,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );

            hit = app
                .models_mut()
                .update(&store, |st| {
                    st.drag_move(
                        window,
                        toast_id,
                        Point::new(Px(x as f32 + 1.0), Px(y as f32)),
                    )
                    .is_some()
                })
                .expect("toast drag follow-up update");

            ui.dispatch_event(
                &mut app,
                &mut services,
                &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                    position: pos,
                    button: fret_core::MouseButton::Left,
                    modifiers: fret_core::Modifiers::default(),
                    is_click: true,
                    click_count: 1,
                    pointer_id: PointerId(0),
                    pointer_type: fret_core::PointerType::Mouse,
                }),
            );

            if hit {
                break;
            }
        }
        if hit {
            break;
        }
    }
    assert!(
        hit,
        "toast did not respond to pointer down; hit@10,10={:?} hit@350,10={:?} hit@500,200={:?}",
        ui.debug_hit_test(Point::new(Px(10.0), Px(10.0))),
        ui.debug_hit_test(Point::new(Px(350.0), Px(10.0))),
        ui.debug_hit_test(Point::new(Px(500.0), Px(200.0))),
    );
}
