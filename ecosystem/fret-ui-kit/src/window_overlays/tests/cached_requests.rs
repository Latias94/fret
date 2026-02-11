use super::*;

#[test]
fn cached_modal_request_is_synthesized_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(1));
    render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
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
            open: open.clone(),
            present: false,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    app.set_frame_id(FrameId(2));
    begin_frame(&mut app, window);
    let _ = app.models_mut().update(&open, |v| *v = true);
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .modals
            .get(&(window, GlobalElementId(0x1)))
            .map(|m| m.layer)
    });
    let layer = layer.expect("modal layer");
    assert!(ui.is_layer_visible(layer));
}

#[test]
fn cached_popover_request_is_synthesized_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let open = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(1));
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );

    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: GlobalElementId(0x2),
            root_name: "popover".into(),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: false,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    app.set_frame_id(FrameId(2));
    begin_frame(&mut app, window);
    let _ = app.models_mut().update(&open, |v| *v = true);
    render(&mut ui, &mut app, &mut services, window, bounds);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .popovers
            .get(&(window, GlobalElementId(0x2)))
            .map(|p| p.layer)
    });
    let layer = layer.expect("popover layer");
    assert!(ui.is_layer_visible(layer));
}

#[test]
fn cached_hover_overlay_request_is_synthesized_for_short_ttl_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let base_open = app.models_mut().insert(false);
    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(0));
    let trigger =
        render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, base_open);

    request_hover_overlay_for_window(
        &mut app,
        window,
        HoverOverlayRequest {
            id: GlobalElementId(0x3),
            root_name: hover_overlay_root_name(GlobalElementId(0x3)),
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
            .get(&(window, GlobalElementId(0x3)))
            .map(|o| o.layer)
    });
    let layer = layer.expect("hover overlay layer");
    assert!(ui.is_layer_visible(layer));

    for _ in 0..OVERLAY_CACHE_TTL_FRAMES {
        begin_frame(&mut app, window);
        render(&mut ui, &mut app, &mut services, window, bounds);
        assert!(
            ui.is_layer_visible(layer),
            "expected hover overlay to remain visible while synthesized from cached request"
        );
    }

    begin_frame(&mut app, window);
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to expire once cache TTL elapses"
    );
}

#[test]
fn cached_tooltip_request_is_synthesized_for_short_ttl_when_open_without_rerender() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    ui.set_window(AppWindowId::default());
    let mut services = FakeServices::default();
    let window = AppWindowId::default();

    let base_open = app.models_mut().insert(false);
    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    app.set_frame_id(FrameId(0));
    let trigger =
        render_base_with_trigger(&mut ui, &mut app, &mut services, window, bounds, base_open);

    request_tooltip_for_window(
        &mut app,
        window,
        TooltipRequest {
            id: GlobalElementId(0x4),
            root_name: tooltip_root_name(GlobalElementId(0x4)),
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
        overlays
            .tooltips
            .get(&(window, GlobalElementId(0x4)))
            .map(|t| t.layer)
    });
    let layer = layer.expect("tooltip layer");
    assert!(ui.is_layer_visible(layer));

    for _ in 0..OVERLAY_CACHE_TTL_FRAMES {
        begin_frame(&mut app, window);
        render(&mut ui, &mut app, &mut services, window, bounds);
        assert!(
            ui.is_layer_visible(layer),
            "expected tooltip to remain visible while synthesized from cached request"
        );
    }

    begin_frame(&mut app, window);
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected tooltip to expire once cache TTL elapses"
    );
}
