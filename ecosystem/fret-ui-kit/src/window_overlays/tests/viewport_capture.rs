use super::*;

#[test]
fn viewport_capture_hides_hover_overlays_and_restores_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base so we can request a hover overlay above it.
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay pointer region.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlay to become visible again after capture release"
    );
}

#[test]
fn viewport_capture_cancel_restores_hover_overlays() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // First frame: render base so we can request a hover overlay above it.
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .hover_overlays
            .get(&(window, trigger))
            .map(|h| h.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay pointer region.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: Some(Point::new(Px(10.0), Px(130.0))),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be cleared after pointer cancel"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: trigger,
            root_name: hover_overlay_root_name(trigger),
            interactive: true,
            trigger,
            open: open.clone(),
            present: true,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlay to become visible again after capture cancel"
    );
}

#[test]
fn viewport_capture_hides_tooltips_and_restores_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Up {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            is_click: false,
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        ui.is_layer_visible(layer),
        "expected tooltip to become visible again after capture release"
    );
}

#[test]
fn viewport_capture_cancel_restores_tooltips() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.tooltips.get(&(window, trigger)).map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    // Start a viewport-like pointer capture by pressing the underlay pointer region.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(0),
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected underlay to capture the pointer"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to be hidden during viewport pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(0),
            position: Some(Point::new(Px(10.0), Px(130.0))),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    assert!(
        ui.captured().is_none(),
        "expected capture to be cleared after pointer cancel"
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: trigger,
            root_name: tooltip_root_name(trigger),
            interactive: true,
            trigger: Some(trigger),
            open: open.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected tooltip to become visible again after capture cancel"
    );
}
