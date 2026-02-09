use super::*;

#[test]
fn dock_drag_closes_non_modal_overlays_for_entire_window() {
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

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a non-modal popover overlay.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
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
            consume_outside_pointer_events: false,
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
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Start a dock drag session for a *different* pointer id (window-global suppression).
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed.
    begin_frame(&mut app, window);
    let _trigger3 = render_base_with_trigger(
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
            consume_outside_pointer_events: false,
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
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dock_drag_closes_dismissible_popovers_only_in_affected_window() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: render base to establish stable bounds for the trigger element in each window.
    let trigger_a = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_b = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 2: open a non-modal popover overlay in both windows.
    let _ = app.models_mut().update(&open_a, |v| *v = true);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(true));
    assert_eq!(app.models().get_copied(&open_b), Some(true));

    // Start a dock drag session for window A only.
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Frame 3: window A popover should be force-closed; window B popover should remain open.
    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(false));
    assert_eq!(
        app.models().get_copied(&open_b),
        Some(true),
        "expected dock drag to only affect overlays in windows participating in the drag session"
    );
}

#[test]
fn dock_drag_cross_window_hides_overlays_in_source_and_current_window() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);
    let underlay_clicked_a = app.models_mut().insert(false);
    let underlay_clicked_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: render base in both windows and show a tooltip + hover overlay above each.
    let (trigger_a, _underlay_a) = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let (trigger_b, _underlay_b) = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    let (hover_layer_a, tooltip_layer_a, hover_layer_b, tooltip_layer_b) = app
        .with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let hover_layer_a = overlays
                .hover_overlays
                .get(&(window_a, trigger_a))
                .map(|h| h.layer);
            let tooltip_layer_a = overlays
                .tooltips
                .get(&(window_a, trigger_a))
                .map(|t| t.layer);
            let hover_layer_b = overlays
                .hover_overlays
                .get(&(window_b, trigger_b))
                .map(|h| h.layer);
            let tooltip_layer_b = overlays
                .tooltips
                .get(&(window_b, trigger_b))
                .map(|t| t.layer);
            (
                hover_layer_a,
                tooltip_layer_a,
                hover_layer_b,
                tooltip_layer_b,
            )
        });
    let hover_layer_a = hover_layer_a.expect("hover overlay layer a");
    let tooltip_layer_a = tooltip_layer_a.expect("tooltip layer a");
    let hover_layer_b = hover_layer_b.expect("hover overlay layer b");
    let tooltip_layer_b = tooltip_layer_b.expect("tooltip layer b");
    assert!(ui_a.is_layer_visible(hover_layer_a));
    assert!(ui_a.is_layer_visible(tooltip_layer_a));
    assert!(ui_b.is_layer_visible(hover_layer_b));
    assert!(ui_b.is_layer_visible(tooltip_layer_b));

    // Frame 2: start a cross-window dock drag in window A; only the source window should be affected
    // until the drag enters window B.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    assert!(
        !ui_a.is_layer_visible(hover_layer_a),
        "expected source window hover overlays to be hidden during dock drag"
    );
    assert!(
        !ui_a.is_layer_visible(tooltip_layer_a),
        "expected source window tooltips to be hidden during dock drag"
    );
    assert!(
        ui_b.is_layer_visible(hover_layer_b),
        "expected non-affected window hover overlays to remain visible before entering the window"
    );
    assert!(
        ui_b.is_layer_visible(tooltip_layer_b),
        "expected non-affected window tooltips to remain visible before entering the window"
    );

    // Frame 3: move the active drag session into window B; both windows should now be affected
    // (source window + current hover window).
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_b;

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    assert!(
        !ui_a.is_layer_visible(hover_layer_a),
        "expected source window hover overlays to remain hidden while dragging across windows"
    );
    assert!(
        !ui_a.is_layer_visible(tooltip_layer_a),
        "expected source window tooltips to remain hidden while dragging across windows"
    );
    assert!(
        !ui_b.is_layer_visible(hover_layer_b),
        "expected current window hover overlays to be hidden while dragging across windows"
    );
    assert!(
        !ui_b.is_layer_visible(tooltip_layer_b),
        "expected current window tooltips to be hidden while dragging across windows"
    );

    // Frame 4: end the drag; both windows should show overlays again if they are re-requested.
    app.cancel_drag(PointerId(7));

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    assert!(
        ui_a.is_layer_visible(hover_layer_a),
        "expected source window hover overlays to be visible again after drag ends"
    );
    assert!(
        ui_a.is_layer_visible(tooltip_layer_a),
        "expected source window tooltips to be visible again after drag ends"
    );
    assert!(
        ui_b.is_layer_visible(hover_layer_b),
        "expected current window hover overlays to be visible again after drag ends"
    );
    assert!(
        ui_b.is_layer_visible(tooltip_layer_b),
        "expected current window tooltips to be visible again after drag ends"
    );
}

#[test]
fn dock_drag_cross_window_leaving_current_window_restores_overlays_in_that_window() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);
    let underlay_clicked_a = app.models_mut().insert(false);
    let underlay_clicked_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: render base in both windows and show a tooltip + hover overlay above each.
    let (trigger_a, _underlay_a) = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let (trigger_b, _underlay_b) = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    let (hover_layer_a, tooltip_layer_a, hover_layer_b, tooltip_layer_b) = app
        .with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let hover_layer_a = overlays
                .hover_overlays
                .get(&(window_a, trigger_a))
                .map(|h| h.layer);
            let tooltip_layer_a = overlays
                .tooltips
                .get(&(window_a, trigger_a))
                .map(|t| t.layer);
            let hover_layer_b = overlays
                .hover_overlays
                .get(&(window_b, trigger_b))
                .map(|h| h.layer);
            let tooltip_layer_b = overlays
                .tooltips
                .get(&(window_b, trigger_b))
                .map(|t| t.layer);
            (
                hover_layer_a,
                tooltip_layer_a,
                hover_layer_b,
                tooltip_layer_b,
            )
        });
    let hover_layer_a = hover_layer_a.expect("hover overlay layer a");
    let tooltip_layer_a = tooltip_layer_a.expect("tooltip layer a");
    let hover_layer_b = hover_layer_b.expect("hover overlay layer b");
    let tooltip_layer_b = tooltip_layer_b.expect("tooltip layer b");
    assert!(ui_a.is_layer_visible(hover_layer_a));
    assert!(ui_a.is_layer_visible(tooltip_layer_a));
    assert!(ui_b.is_layer_visible(hover_layer_b));
    assert!(ui_b.is_layer_visible(tooltip_layer_b));

    // Frame 2: enter window B as the current hover window; both windows are affected.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_b;

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    assert!(!ui_a.is_layer_visible(hover_layer_a));
    assert!(!ui_a.is_layer_visible(tooltip_layer_a));
    assert!(!ui_b.is_layer_visible(hover_layer_b));
    assert!(!ui_b.is_layer_visible(tooltip_layer_b));

    // Frame 3: drag leaves window B (current window returns to source window); window B should
    // restore overlays while window A remains affected.
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_a;

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
        underlay_clicked_a.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_a,
        HoverOverlayRequest {
            id: trigger_a,
            root_name: hover_overlay_root_name(trigger_a),
            interactive: true,
            trigger: trigger_a,
            open: open_a.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_a,
        TooltipRequest {
            id: trigger_a,
            root_name: tooltip_root_name(trigger_a),
            interactive: true,
            trigger: Some(trigger_a),
            open: open_a.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
        underlay_clicked_b.clone(),
    );
    request_hover_overlay_for_window(
        &mut app,
        window_b,
        HoverOverlayRequest {
            id: trigger_b,
            root_name: hover_overlay_root_name(trigger_b),
            interactive: true,
            trigger: trigger_b,
            open: open_b.clone(),
            present: true,
            children: Vec::new(),
        },
    );
    request_tooltip_for_window(
        &mut app,
        window_b,
        TooltipRequest {
            id: trigger_b,
            root_name: tooltip_root_name(trigger_b),
            interactive: true,
            trigger: Some(trigger_b),
            open: open_b.clone(),
            present: true,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);

    assert!(
        !ui_a.is_layer_visible(hover_layer_a),
        "expected source window overlays to remain hidden while drag continues"
    );
    assert!(
        !ui_a.is_layer_visible(tooltip_layer_a),
        "expected source window overlays to remain hidden while drag continues"
    );
    assert!(
        ui_b.is_layer_visible(hover_layer_b),
        "expected hover window overlays to restore once the drag leaves the window"
    );
    assert!(
        ui_b.is_layer_visible(tooltip_layer_b),
        "expected hover window overlays to restore once the drag leaves the window"
    );

    app.cancel_drag(PointerId(7));
}

#[test]
fn dock_drag_cross_window_closes_dismissible_popovers_in_source_and_current_window() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: establish stable trigger ids.
    let trigger_a = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_b = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 2: open a dismissible popover in both windows.
    let _ = app.models_mut().update(&open_a, |v| *v = true);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(true));
    assert_eq!(app.models().get_copied(&open_b), Some(true));

    // Frame 3: start a cross-window dock drag from window A.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Source window should be affected immediately.
    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open_a), Some(false));

    // Hover window is not affected until the drag enters it.
    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(app.models().get_copied(&open_b), Some(true));

    // Frame 4: simulate the drag entering window B; window B should now be affected as well.
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_b;

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open_b),
        Some(false),
        "expected popovers in the current hover window to close once the drag enters it"
    );
}

#[test]
fn dock_drag_cross_window_closes_menu_like_overlays_and_clears_occlusion() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: establish stable trigger ids.
    let trigger_a = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_b = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 2: open menu-like overlays in both windows (pointer occlusion).
    let _ = app.models_mut().update(&open_a, |v| *v = true);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap_a = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui_a, &mut app, window_a,
    );
    let snap_b = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui_b, &mut app, window_b,
    );
    assert_eq!(app.models().get_copied(&open_a), Some(true));
    assert_eq!(app.models().get_copied(&open_b), Some(true));
    assert_eq!(
        snap_a.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll
    );
    assert_eq!(
        snap_b.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll
    );

    // Frame 3: start a cross-window dock drag and enter window B; both windows should drop occlusion.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_b;

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap_a = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui_a, &mut app, window_a,
    );
    let snap_b = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui_b, &mut app, window_b,
    );
    assert_eq!(app.models().get_copied(&open_a), Some(false));
    assert_eq!(app.models().get_copied(&open_b), Some(false));
    assert_eq!(
        snap_a.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert_eq!(
        snap_b.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert_eq!(snap_a.topmost_pointer_occluding_overlay, None);
    assert_eq!(snap_b.topmost_pointer_occluding_overlay, None);
}

#[test]
fn dock_drag_cross_window_leaving_current_window_does_not_restore_closed_popovers() {
    use slotmap::KeyData;

    let window_a = AppWindowId::from(KeyData::from_ffi(1));
    let window_b = AppWindowId::from(KeyData::from_ffi(2));

    let mut app = App::new();

    let mut ui_a: UiTree<App> = UiTree::new();
    ui_a.set_window(window_a);
    let mut ui_b: UiTree<App> = UiTree::new();
    ui_b.set_window(window_b);

    let open_a = app.models_mut().insert(false);
    let open_b = app.models_mut().insert(false);

    let mut services = FakeServices;
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(300.0), Px(200.0)),
    );

    // Frame 1: establish stable trigger ids.
    let trigger_a = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    let trigger_b = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    // Frame 2: open a dismissible popover in both windows.
    let _ = app.models_mut().update(&open_a, |v| *v = true);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(true));
    assert_eq!(app.models().get_copied(&open_b), Some(true));

    // Frame 3: start a cross-window dock drag and enter window B; both windows should close popovers.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window_a,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_b;

    begin_frame(&mut app, window_a);
    let _ = render_base_with_trigger(
        &mut ui_a,
        &mut app,
        &mut services,
        window_a,
        bounds,
        open_a.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_a,
        DismissiblePopoverRequest {
            id: trigger_a,
            root_name: popover_root_name(trigger_a),
            trigger: trigger_a,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_a, &mut app, &mut services, window_a, bounds);
    ui_a.layout_all(&mut app, &mut services, bounds, 1.0);

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(app.models().get_copied(&open_a), Some(false));
    assert_eq!(app.models().get_copied(&open_b), Some(false));

    // Frame 4: drag leaves window B; the previously closed popover should remain closed.
    let drag = app.drag_mut(PointerId(7)).expect("drag session");
    drag.current_window = window_a;

    begin_frame(&mut app, window_b);
    let _ = render_base_with_trigger(
        &mut ui_b,
        &mut app,
        &mut services,
        window_b,
        bounds,
        open_b.clone(),
    );
    request_dismissible_popover_for_window(
        &mut app,
        window_b,
        DismissiblePopoverRequest {
            id: trigger_b,
            root_name: popover_root_name(trigger_b),
            trigger: trigger_b,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: Vec::new(),
        },
    );
    render(&mut ui_b, &mut app, &mut services, window_b, bounds);
    ui_b.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        app.models().get_copied(&open_b),
        Some(false),
        "expected the popover to remain closed after leaving the window"
    );
}

#[test]
fn dock_drag_hides_hover_overlays_in_affected_window() {
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

    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
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
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        !ui.is_layer_visible(layer),
        "expected dock drag to hide hover overlays in the affected window"
    );

    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked,
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
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);

    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlays to become visible again after dock drag ends"
    );
}

#[test]
fn dock_drag_hides_tooltips_in_affected_window() {
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

    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
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
        "expected dock drag to hide tooltips in the affected window"
    );

    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger_and_underlay(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        underlay_clicked,
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
        "expected tooltips to become visible again after dock drag ends"
    );
}

#[test]
fn dock_drag_forces_menu_like_overlay_to_drop_pointer_occlusion_while_closing() {
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

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a menu-like overlay that enables pointer occlusion.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _trigger2 = render_base_with_trigger(
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

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
        "expected menu-like overlay to enable pointer occlusion while open"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, Some(trigger));

    // Start a dock drag session for a *different* pointer id (window-global suppression).
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed and
    // drop pointer occlusion even if the layer remains present.
    begin_frame(&mut app, window);
    let _trigger3 = render_base_with_trigger(
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

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None,
        "expected dock drag to force menu-like overlay to drop pointer occlusion"
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, None);
}

#[test]
fn dock_drag_closes_menu_like_overlay_and_disables_pointer_move_observers() {
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

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a menu-like overlay that requests pointer-move observers (submenu safe-corridor).
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    let on_pointer_move: fret_ui::action::OnDismissiblePointerMove =
        Arc::new(|_host, _cx, _mv| false);
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
            on_pointer_move: Some(on_pointer_move),
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays.popovers.get(&(window, trigger)).map(|p| p.layer)
    });
    let layer = layer.expect("popover layer");

    let info = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
        .expect("popover debug layer info");
    assert!(info.visible);
    assert!(info.wants_pointer_move_events);

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(true));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
        "expected menu-like overlay to enable pointer occlusion while open"
    );

    // Start a dock drag session for a *different* pointer id (window-global suppression).
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; policy should force it closed and drop occlusion.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    let on_pointer_move: fret_ui::action::OnDismissiblePointerMove =
        Arc::new(|_host, _cx, _mv| false);
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
            on_pointer_move: Some(on_pointer_move),
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let snap = crate::overlay_controller::OverlayController::stack_snapshot_for_window(
        &ui, &mut app, window,
    );
    assert_eq!(app.models().get_copied(&open), Some(false));
    assert_eq!(
        snap.arbitration.pointer_occlusion,
        fret_ui::tree::PointerOcclusion::None
    );
    assert_eq!(snap.topmost_pointer_occluding_overlay, None);

    // If the overlay remains present for a close transition, it must not keep requesting observers.
    if let Some(info) = ui
        .debug_layers_in_paint_order()
        .into_iter()
        .find(|l| l.id == layer)
    {
        assert!(!info.hit_testable);
        assert!(!info.wants_pointer_move_events);
        assert!(!info.wants_pointer_down_outside_events);
    }
}

#[test]
fn dock_drag_does_not_restore_closed_non_modal_overlays_on_drag_end() {
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

    // First frame: render base to establish stable bounds for the trigger element.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Second frame: open a non-modal popover overlay.
    let _ = app.models_mut().update(&open, |v| *v = true);
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
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
            consume_outside_pointer_events: false,
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
    assert_eq!(app.models().get_copied(&open), Some(true));

    // Start a dock drag session; policy should force-close the non-modal overlay.
    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );

    // Third frame: re-request the overlay; window_overlays policy should force it closed.
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
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
            consume_outside_pointer_events: false,
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
    assert_eq!(app.models().get_copied(&open), Some(false));

    // End the drag and render another frame; overlays should stay closed unless the user reopens.
    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
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
            consume_outside_pointer_events: false,
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
    assert_eq!(app.models().get_copied(&open), Some(false));
}

#[test]
fn dock_drag_restores_focus_when_focus_is_missing_on_drag_end() {
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

    // First frame: render base and focus the trigger.
    let trigger = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let trigger_node =
        fret_ui::elements::node_for_element(&mut app, window, trigger).expect("trigger node");
    ui.set_focus(Some(trigger_node));

    // Start a dock drag session and render a frame so the overlay policy can record the focus snapshot.
    app.begin_cross_window_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
    );
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    // Simulate focus being cleared during the drag (platform/runner behavior).
    ui.set_focus(None);

    // End the drag and render another frame; focus should restore to the pre-drag focus node.
    app.cancel_drag(PointerId(7));
    begin_frame(&mut app, window);
    let _ = render_base_with_trigger(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(ui.focus(), Some(trigger_node));
}

#[test]
fn dock_drag_keeps_hover_overlays_hidden_after_capture_release_until_drag_ends() {
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

    // Frame 1: render base + show a hover overlay.
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

    // Frame 2: capture the pointer (viewport-like capture) and start a dock drag session.
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
    assert!(ui.captured().is_some(), "expected pointer capture");

    app.begin_drag_with_kind(
        PointerId(7),
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        Point::new(Px(10.0), Px(10.0)),
        (),
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
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to be hidden during capture + dock drag"
    );

    // Frame 3: release capture; dock drag remains active so overlays should stay hidden.
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
    assert!(ui.captured().is_none(), "expected capture release");

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
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        !ui.is_layer_visible(layer),
        "expected hover overlay to remain hidden while dock drag is active"
    );

    // Frame 4: end the drag; overlays can become visible again when re-requested.
    app.cancel_drag(PointerId(7));

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
            children: Vec::new(),
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    assert!(
        ui.is_layer_visible(layer),
        "expected hover overlay to become visible again after dock drag ends"
    );
}
