use super::*;

#[test]
fn tooltip_is_pointer_transparent_and_does_not_request_observers_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let base_open = app.models_mut().insert(false);
    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, base_open);

    // Install a tooltip layer that is still `present` but `open=false` (closing animation).
    begin_frame(&mut app, window);
    let tooltip_id = GlobalElementId(0x44);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: tooltip_id,
            root_name: tooltip_root_name(tooltip_id),
            interactive: true,
            trigger: None,
            open,
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays
                .tooltips
                .get(&(window, tooltip_id))
                .map(|p| p.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert_eq!(
        info.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert!(
        !info.wants_pointer_down_outside_events,
        "expected tooltip to stop requesting outside-press observers during close transitions"
    );
    assert!(
        !info.wants_pointer_move_events,
        "expected tooltip to stop requesting pointer-move observers during close transitions"
    );
}

#[test]
fn tooltip_does_not_request_observers_by_default() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let _open = app.models_mut().insert(true);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Tooltips are click-through and should not install outside-press / pointer-move observers
    // unless the request explicitly opts into them.
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    let open = app.models_mut().insert(true);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id,
            root_name: tooltip_root_name(id),
            interactive: true,
            trigger: Some(id),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.tooltips.get(&(window, id)).map(|t| t.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}

#[test]
fn tooltip_does_not_request_observers_while_closing() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let _open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Base root (required so the window exists and rendering can proceed).
    begin_frame(&mut app, window);
    let base = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "base",
        |_| Vec::new(),
    );
    ui.set_root(base);

    // Install a tooltip layer that is still present but non-interactive (closing animation).
    begin_frame(&mut app, window);
    let id = GlobalElementId(0xdead);
    let open = app.models_mut().insert(false);
    let handler: fret_ui::action::OnDismissRequest = Arc::new(|_host, _cx, _req| {});
    let on_pointer_move: fret_ui::action::OnDismissiblePointerMove =
        Arc::new(|_host, _cx, _move| false);
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id,
            root_name: tooltip_root_name(id),
            interactive: false,
            trigger: Some(id),
            open: open.clone(),
            present: true,
            on_dismiss_request: Some(handler),
            on_pointer_move: Some(on_pointer_move),
            children: Vec::new(),
        },
    );

    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            overlays.tooltips.get(&(window, id)).map(|t| t.layer)
        })
        .expect("tooltip layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("tooltip debug layer info");

    assert!(info.visible);
    assert!(!info.blocks_underlay_input);
    assert!(!info.hit_testable);
    assert!(!info.wants_pointer_down_outside_events);
    assert!(!info.wants_pointer_move_events);
}
