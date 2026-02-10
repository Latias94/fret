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
        ToastLayerRequest::new(viewport_id, store.clone())
            .position(ToastPosition::BottomCenter)
            .style({
                let mut style = ToastLayerStyle::default();
                style.open_ticks = 0;
                style.close_ticks = 0;
                style.slide_distance = Px(0.0);
                style
            }),
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
fn toast_layer_container_aria_label_is_exposed_in_semantics_snapshot() {
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
        ToastLayerRequest::new(viewport_id, store.clone()).container_aria_label("Notifications"),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    assert!(
        snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::Viewport
                && n.label.as_deref() == Some("Notifications")
        }),
        "expected a toast viewport semantics node with label=Notifications",
    );
}

#[test]
fn toast_layer_custom_aria_label_overrides_container_label() {
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
        ToastLayerRequest::new(viewport_id, store.clone())
            .container_aria_label("Notifications")
            .custom_aria_label_opt(Some(Arc::from("Custom notifications label"))),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    assert!(
        snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::Viewport
                && n.label.as_deref() == Some("Custom notifications label")
        }),
        "expected a toast viewport semantics node with the custom label",
    );
}

#[test]
fn toast_layer_close_button_has_a11y_label() {
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
        ToastLayerRequest::new(viewport_id, store.clone()).style({
            let mut style = ToastLayerStyle::default();
            style.open_ticks = 0;
            style.close_ticks = 0;
            style.slide_distance = Px(0.0);
            style
        }),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    assert!(
        snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("Close toast")
        }),
        "expected a close button semantics node with label=Close toast",
    );
}

#[test]
fn toast_request_can_disable_close_button() {
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
        ToastRequest::new("Hello").close_button(false),
    );

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone()).style({
            let mut style = ToastLayerStyle::default();
            style.open_ticks = 0;
            style.close_ticks = 0;
            style.slide_distance = Px(0.0);
            style
        }),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    assert!(
        !snap.nodes.iter().any(|n| {
            n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("Close toast")
        }),
        "expected no close button semantics nodes when toast.close_button=false",
    );
}

#[test]
fn toast_layer_mobile_offset_defaults_to_16px_at_600px_breakpoint() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(600.0), Px(200.0)),
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
        ToastRequest::new("Hello").test_id("toast-mobile-default-offset"),
    );

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone()),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    let toast = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("toast-mobile-default-offset"))
        .expect("toast semantics node");

    assert!(
        (toast.bounds.origin.x.0 - 16.0).abs() <= 1.0,
        "expected toast x≈16 got={}",
        toast.bounds.origin.x.0
    );
    assert!(
        (toast.bounds.size.width.0 - (600.0 - 16.0 - 16.0)).abs() <= 2.0,
        "expected toast w≈{} got={}",
        600.0 - 16.0 - 16.0,
        toast.bounds.size.width.0
    );
}

#[test]
fn toast_layer_mobile_offset_can_override_left_and_right() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(600.0), Px(200.0)),
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
        ToastRequest::new("Hello").test_id("toast-mobile-custom-offset"),
    );

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone())
            .position(ToastPosition::BottomCenter)
            .style({
                let mut style = ToastLayerStyle::default();
                style.open_ticks = 0;
                style.close_ticks = 0;
                style.slide_distance = Px(0.0);
                style
            })
            .mobile_offset(
                ToastOffset::default()
                    .left(Px(10.0))
                    .right(Px(20.0))
                    .top(Px(12.0))
                    .bottom(Px(12.0)),
            ),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");

    let toast = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("toast-mobile-custom-offset"))
        .expect("toast semantics node");

    assert!(
        (toast.bounds.origin.x.0 - 10.0).abs() <= 1.0,
        "expected toast x≈10 got={}",
        toast.bounds.origin.x.0
    );
    assert!(
        (toast.bounds.size.width.0 - (600.0 - 10.0 - 20.0)).abs() <= 2.0,
        "expected toast w≈{} got={}",
        600.0 - 10.0 - 20.0,
        toast.bounds.size.width.0
    );
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
            let started = st.begin_drag(
                window,
                GlobalElementId(0xbeef),
                toast_id,
                Point::new(Px(0.0), Px(0.0)),
                ToastPosition::TopLeft,
            );
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

#[test]
fn toast_layer_stays_above_modal_even_when_installed_first() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices::default();

    fret_runtime::apply_window_metrics_event(&mut app, window, &Event::WindowFocusChanged(true));

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

    // Frame A: create the toast layer first (simulating a long-lived app-mounted toaster).
    begin_frame(&mut app, window);
    let toaster_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(toaster_id, store.clone())
            .position(ToastPosition::TopLeft)
            .margin(Px(0.0))
            .gap(Px(0.0)),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame B: open a modal after the toast layer already exists.
    let modal_open = app.models_mut().insert(true);
    begin_frame(&mut app, window);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(toaster_id, store.clone())
            .position(ToastPosition::TopLeft)
            .margin(Px(0.0))
            .gap(Px(0.0)),
    );
    request_modal_for_window(
        &mut app,
        window,
        ModalRequest {
            id: GlobalElementId(0x1),
            root_name: "modal".into(),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: modal_open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (toast_layer, modal_layer) =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let toast_layer = overlays
                .toast_layers
                .get(&(window, toaster_id))
                .map(|entry| entry.layer)
                .expect("toast layer installed");
            let modal_layer = overlays
                .modals
                .get(&(window, GlobalElementId(0x1)))
                .map(|entry| entry.layer)
                .expect("modal layer installed");
            (toast_layer, modal_layer)
        });

    let order = ui.layer_ids_in_paint_order();
    let toast_index = order
        .iter()
        .position(|&id| id == toast_layer)
        .expect("toast layer in order");
    let modal_index = order
        .iter()
        .position(|&id| id == modal_layer)
        .expect("modal layer in order");
    assert!(
        toast_index > modal_index,
        "expected toast layer to be above modal: order={:?}",
        ui.debug_layers_in_paint_order()
            .into_iter()
            .map(|l| (l.id, l.blocks_underlay_input, l.hit_testable, l.visible))
            .collect::<Vec<_>>()
    );

    // Prove the toast is still interactive above the modal barrier by finding a point that starts
    // a swipe drag (which requires pointer-down to hit the toast subtree).
    let mut hit = false;
    for y in (8..=160).step_by(16) {
        for x in (8..=240).step_by(16) {
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
        "toast did not respond to pointer down above modal barrier; hit@10,10={:?} hit@500,10={:?}",
        ui.debug_hit_test(Point::new(Px(10.0), Px(10.0))),
        ui.debug_hit_test(Point::new(Px(500.0), Px(10.0))),
    );
}

#[test]
fn toast_async_queue_upsert_is_drained_during_render() {
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
    let queue = toast_async_queue(&mut app);
    queue.upsert(window, ToastRequest::new("Hello from async queue"));

    begin_frame(&mut app, window);
    let viewport_id = GlobalElementId(0xbeef);
    request_toast_layer_for_window(
        &mut app,
        window,
        ToastLayerRequest::new(viewport_id, store.clone()),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let toast_count = app
        .models()
        .read(&store, |st| st.toasts_for_window(window).len())
        .unwrap_or_default();
    assert!(toast_count > 0);
}
