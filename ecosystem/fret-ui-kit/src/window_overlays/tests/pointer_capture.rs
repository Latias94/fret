use super::*;

#[test]
fn pointer_capture_forces_menu_like_overlay_to_close_and_drop_occlusion() {
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

    // First frame: render base and capture pointer 0 in the underlay (viewport-like capture).
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.any_captured_node().is_some(),
        "expected pointer capture to be active before opening the menu-like overlay"
    );

    // Second frame: attempt to open a menu-like overlay that would normally enable pointer occlusion.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
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
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected capture to suspend pointer gating (not force-close) for menu-like overlays"
    );

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected pointer occlusion to be suppressed while capture is active"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, None);

    let base_root = ui.base_root().expect("base root");
    let popover_layer = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.root != base_root)
        .expect("popover layer");
    assert!(
        !popover_layer.hit_testable,
        "expected capture to suspend popover pointer hit-testing"
    );
}

#[test]
fn pointer_capture_forces_consuming_popover_to_close() {
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

    // First frame: render base and capture pointer 0 in the underlay (viewport-like capture).
    let (trigger, _underlay) = render_base_with_trigger_and_capture_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.any_captured_node().is_some(),
        "expected pointer capture to be active before opening the consuming popover"
    );

    // Second frame: attempt to open a consuming non-modal overlay. Even without pointer occlusion,
    // we must not introduce non-click-through dismissal semantics while another layer owns capture.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_capture_underlay(
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
            id: trigger,
            root_name: popover_root_name(trigger),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open),
        Some(true),
        "expected pointer capture to suspend pointer gating (not force-close) for consuming popovers"
    );

    let base_root = ui.base_root().expect("base root");
    let popover_layer = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.root != base_root)
        .expect("popover layer");
    assert!(
        !popover_layer.hit_testable,
        "expected capture to suspend popover pointer hit-testing"
    );
}

#[test]
fn pointer_capture_hides_hover_overlays_in_same_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
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

    // Start a pointer-capture session by pressing (without releasing) a `Pressable`.
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
        "expected hover overlay to be hidden during pointer capture"
    );
}

#[test]
fn pointer_capture_restores_hover_overlays_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
        "expected hover overlay to be hidden during pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
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
        "expected pointer capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let (_trigger3, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
fn pointer_capture_hides_tooltips_in_same_window() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
        "expected tooltip to be hidden during pointer capture"
    );
}

#[test]
fn pointer_capture_restores_tooltips_after_release() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);

    let open = app.models_mut().insert(false);
    let underlay_clicked = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    let (trigger, _underlay) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
    );
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

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(10.0), Px(130.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            click_count: 1,
            pointer_id: PointerId(0),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(
        ui.captured().is_some(),
        "expected pressable pointer down to capture"
    );

    begin_frame(&mut app, window);
    let (_trigger2, _underlay2) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
        "expected tooltip to be hidden during pointer capture"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
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
        "expected pointer capture to be released on pointer up"
    );

    begin_frame(&mut app, window);
    let (_trigger3, _underlay3) = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked.clone(),
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
fn pointer_capture_multiple_roots_hides_hover_overlays_and_tooltips() {
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

    // Frame 1: render base and request overlays so we can track layer visibility.
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
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let (hover_layer, tooltip_layer) =
        app.with_global_mut_untracked(WindowOverlays::default, |o, _| {
            let hover_layer = o
                .hover_overlays
                .get(&(window, trigger))
                .map(|h| h.layer)
                .expect("hover overlay layer");
            let tooltip_layer = o
                .tooltips
                .get(&(window, trigger))
                .map(|t| t.layer)
                .expect("tooltip layer");
            (hover_layer, tooltip_layer)
        });
    assert!(ui.is_layer_visible(hover_layer));
    assert!(ui.is_layer_visible(tooltip_layer));

    // Begin capture in the base layer (viewport-like) and in the foreign overlay layer (second pointer).
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
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected pointer 0 capture to start from the base underlay"
    );

    // Add a foreign overlay layer that can independently capture a separate pointer id.
    let foreign_overlay_root = fret_ui::declarative::render_dismissible_root_with_hooks(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "foreign-capture-overlay",
        |cx| {
            vec![cx.pointer_region(
                PointerRegionProps {
                    layout: {
                        LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                left: Some(Px(200.0)),
                                top: Some(Px(0.0)),
                                ..Default::default()
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(40.0)),
                                height: Length::Px(Px(40.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    },
                    enabled: true,
                },
                |cx| {
                    cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                        host.capture_pointer();
                        true
                    }));
                    Vec::new()
                },
            )]
        },
    );
    ui.push_overlay_root_ex(foreign_overlay_root, false, true);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId(1),
            position: Point::new(Px(210.0), Px(10.0)),
            button: fret_core::MouseButton::Left,
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            click_count: 1,
        }),
    );

    assert!(
        ui.captured_for(fret_core::PointerId(0)).is_some(),
        "expected pointer 0 capture to be active"
    );
    assert!(
        ui.captured_for(fret_core::PointerId(1)).is_some(),
        "expected pointer 1 capture to be active"
    );

    let arbitration = ui.input_arbitration_snapshot();
    assert!(arbitration.pointer_capture_active);
    assert!(
        arbitration.pointer_capture_multiple_layers,
        "expected multiple pointer capture roots across layers"
    );

    // Next frame: overlays should be hidden while multiple capture roots are active.
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

    assert!(!ui.is_layer_visible(hover_layer));
    assert!(!ui.is_layer_visible(tooltip_layer));

    // Cancel both pointers; overlays should be able to become visible again when re-requested.
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
    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::PointerCancel(fret_core::PointerCancelEvent {
            pointer_id: fret_core::PointerId(1),
            position: Some(Point::new(Px(210.0), Px(10.0))),
            buttons: fret_core::MouseButtons::default(),
            modifiers: fret_core::Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    assert!(ui.any_captured_node().is_none());

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

    assert!(ui.is_layer_visible(hover_layer));
    assert!(ui.is_layer_visible(tooltip_layer));
}
