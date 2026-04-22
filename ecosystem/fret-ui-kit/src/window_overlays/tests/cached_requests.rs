use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

fn zero_motion_toast_style() -> ToastLayerStyle {
    let mut style = ToastLayerStyle::default();
    style.open_ticks = 0;
    style.close_ticks = 0;
    style.slide_distance = Px(0.0);
    style
}

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
fn hidden_popover_finalizer_does_not_restore_focus_over_same_frame_overlay_handoff() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open_a = app.models_mut().insert(true);
    let open_b = app.models_mut().insert(false);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(220.0), Px(140.0)),
    );

    let popover_a_id = GlobalElementId(0x20);
    let popover_b_id = GlobalElementId(0x21);
    let mut trigger_a: Option<GlobalElementId> = None;
    let mut trigger_b: Option<GlobalElementId> = None;
    let mut popover_a_focus: Option<GlobalElementId> = None;
    let mut popover_b_focus: Option<GlobalElementId> = None;

    begin_frame(&mut app, window);
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "cached-popover-handoff",
        |cx| {
            vec![
                cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_a = Some(id);
                        Vec::new()
                    },
                ),
                cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_b = Some(id);
                        Vec::new()
                    },
                ),
            ]
        },
    );
    ui.set_root(root);

    let overlay_a_children = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "cached-popover-handoff.a",
        |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(96.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    popover_a_focus = Some(id);
                    Vec::new()
                },
            )]
        },
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: popover_a_id,
            root_name: "cached-popover-handoff.a".into(),
            trigger: trigger_a.expect("trigger a"),
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_a.clone(),
            present: true,
            initial_focus: popover_a_focus,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_a_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let popover_a_focus_node = fret_ui::elements::node_for_element(
        &mut app,
        window,
        popover_a_focus.expect("popover a focus"),
    )
    .expect("popover a focus node");
    assert_eq!(ui.focus(), Some(popover_a_focus_node));

    let _ = app.models_mut().update(&open_a, |v| *v = false);
    let _ = app.models_mut().update(&open_b, |v| *v = true);

    begin_frame(&mut app, window);
    trigger_a = None;
    trigger_b = None;
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "cached-popover-handoff",
        |cx| {
            vec![
                cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_a = Some(id);
                        Vec::new()
                    },
                ),
                cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_b = Some(id);
                        Vec::new()
                    },
                ),
            ]
        },
    );
    ui.set_root(root);

    let overlay_b_children = fret_ui::elements::with_element_cx(
        &mut app,
        window,
        bounds,
        "cached-popover-handoff.b",
        |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(96.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    popover_b_focus = Some(id);
                    Vec::new()
                },
            )]
        },
    );
    request_dismissible_popover_for_window(
        &mut app,
        window,
        DismissiblePopoverRequest {
            id: popover_b_id,
            root_name: "cached-popover-handoff.b".into(),
            trigger: trigger_b.expect("trigger b"),
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: true,
            disable_outside_pointer_events: true,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: open_b.clone(),
            present: true,
            initial_focus: popover_b_focus,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: overlay_b_children,
        },
    );
    render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let popover_b_focus_node = fret_ui::elements::node_for_element(
        &mut app,
        window,
        popover_b_focus.expect("popover b focus"),
    )
    .expect("popover b focus node");
    assert_eq!(ui.focus(), Some(popover_b_focus_node));
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

#[test]
fn owned_cached_modal_request_is_pruned_when_request_owner_is_removed() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let modal_id = GlobalElementId(0x41);
    let mut show_owner = true;
    let mut modal_layer: Option<fret_ui::tree::UiLayerId> = None;
    let mut recorded_owner: Option<GlobalElementId> = None;

    for _frame in 0..2 {
        begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-modal-prune",
            |cx| {
                let mut out = Vec::new();
                if show_owner {
                    out.push(cx.keyed("owner", |cx| {
                        crate::OverlayController::request(
                            cx,
                            crate::OverlayRequest::modal(
                                modal_id,
                                None,
                                open.clone(),
                                crate::OverlayPresence::instant(true),
                                Vec::new(),
                            ),
                        );
                        cx.text("owner")
                    }));
                }
                out
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        if show_owner {
            modal_layer =
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    recorded_owner = overlays
                        .cached_modal_requests
                        .get(&(window, modal_id))
                        .and_then(|entry| entry.owner);
                    overlays
                        .modals
                        .get(&(window, modal_id))
                        .map(|entry| entry.layer)
                });
            let layer = modal_layer.expect("modal layer");
            assert!(ui.is_layer_visible(layer));
            assert!(
                recorded_owner.is_some(),
                "owned overlay requests should retain their declarative owner while the producer is still rendered"
            );
            show_owner = false;
        }
    }

    let layer = modal_layer.expect("modal layer");
    assert!(
        ui.layer_root(layer).is_none(),
        "owned overlay layer should be removed once its request owner disappears"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays.modals.contains_key(&(window, modal_id)),
            "active modal entry should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_modal_requests
                .contains_key(&(window, modal_id)),
            "cached modal declaration should be pruned with the removed owner"
        );
    });
}

#[test]
fn owned_cached_modal_prune_runs_close_auto_focus_before_teardown() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let modal_id = GlobalElementId(0x4a);
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_handler = calls.clone();
    let underlay_for_handler: Arc<Mutex<Option<GlobalElementId>>> = Arc::new(Mutex::new(None));
    let underlay_for_request = underlay_for_handler.clone();
    let mut underlay_id: Option<GlobalElementId> = None;
    let mut focusable_id: Option<GlobalElementId> = None;

    let on_close_auto_focus: fret_ui::action::OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        calls_for_handler.fetch_add(1, Ordering::SeqCst);
        let target = *underlay_for_handler
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(target) = target {
            host.request_focus(target);
        }
        req.prevent_default();
    });

    // Frame 1: mount an owned modal and focus inside it.
    begin_frame(&mut app, window);
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "owned-cached-modal-close-autofocus-prune",
        |cx| {
            let mut out = vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    underlay_id = Some(id);
                    *underlay_for_request
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(id);
                    Vec::new()
                },
            )];

            let trigger = underlay_id.expect("underlay id");
            let handler = on_close_auto_focus.clone();
            out.push(cx.keyed("owner", |cx| {
                let focusable = cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(96.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        focusable_id = Some(id);
                        Vec::new()
                    },
                );
                let mut req = crate::OverlayRequest::modal(
                    modal_id,
                    Some(trigger),
                    open.clone(),
                    crate::OverlayPresence::instant(true),
                    vec![focusable],
                );
                req.on_close_auto_focus = Some(handler);
                crate::OverlayController::request(cx, req);
                cx.text("owner")
            }));

            out
        },
    );
    ui.set_root(root);
    crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let focusable = focusable_id.expect("modal focusable");
    let focusable_node =
        fret_ui::elements::node_for_element(&mut app, window, focusable).expect("focusable node");
    ui.set_focus(Some(focusable_node));

    // Frame 2: producer disappears while the modal closes; owner-prune should still honor the
    // close auto-focus contract before tearing the layer down.
    let _ = app.models_mut().update(&open, |v| *v = false);
    begin_frame(&mut app, window);
    underlay_id = None;
    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "owned-cached-modal-close-autofocus-prune",
        |cx| {
            vec![cx.pressable_with_id(
                PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Px(Px(80.0));
                        layout.size.height = Length::Px(Px(32.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st, id| {
                    underlay_id = Some(id);
                    *underlay_for_request
                        .lock()
                        .unwrap_or_else(|e| e.into_inner()) = Some(id);
                    Vec::new()
                },
            )]
        },
    );
    ui.set_root(root);
    crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let underlay = underlay_id.expect("underlay id after prune");
    let underlay_node =
        fret_ui::elements::node_for_element(&mut app, window, underlay).expect("underlay node");
    assert!(
        calls.load(Ordering::SeqCst) > 0,
        "expected owner-pruned modal teardown to run close auto-focus"
    );
    assert_eq!(
        ui.focus(),
        Some(underlay_node),
        "expected close auto-focus to redirect focus before the pruned modal layer is removed"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays.modals.contains_key(&(window, modal_id)),
            "active modal entry should be removed after owner-prune close handling"
        );
        assert!(
            !overlays
                .cached_modal_requests
                .contains_key(&(window, modal_id)),
            "cached modal declaration should be removed after owner-prune close handling"
        );
    });
}

#[test]
fn owned_cached_hover_request_is_pruned_immediately_when_request_owner_is_removed() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let hover_id = GlobalElementId(0x42);
    let mut show_owner = true;
    let mut hover_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..2 {
        begin_frame(&mut app, window);
        let mut trigger_id: Option<GlobalElementId> = None;
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-hover-prune",
            |cx| {
                let mut out = vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_id = Some(id);
                        Vec::new()
                    },
                )];

                if show_owner {
                    let trigger = trigger_id.expect("trigger id");
                    out.push(cx.keyed("owner", |cx| {
                        crate::OverlayController::request(
                            cx,
                            crate::OverlayRequest::hover(
                                hover_id,
                                trigger,
                                open.clone(),
                                crate::OverlayPresence::instant(true),
                                Vec::new(),
                            ),
                        );
                        cx.text("owner")
                    }));
                }

                out
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        if show_owner {
            hover_layer =
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    overlays
                        .hover_overlays
                        .get(&(window, hover_id))
                        .map(|entry| entry.layer)
                });
            let layer = hover_layer.expect("hover layer");
            assert!(ui.is_layer_visible(layer));
            show_owner = false;
        }
    }

    let layer = hover_layer.expect("hover layer");
    assert!(
        ui.layer_root(layer).is_none(),
        "hover overlay layer should be removed immediately once its request owner disappears"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays.hover_overlays.contains_key(&(window, hover_id)),
            "active hover entry should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_hover_overlay_requests
                .contains_key(&(window, hover_id)),
            "cached hover declaration should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_hover_overlay_pointer_move_handlers
                .contains_key(&(window, hover_id)),
            "cached hover pointer-move handler should be pruned with the removed owner"
        );
    });
}

#[test]
fn owned_cached_modal_request_stays_visible_during_view_cache_reuse() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let modal_id = GlobalElementId(0x43);
    let renders = Arc::new(AtomicUsize::new(0));
    let mut modal_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..4 {
        begin_frame(&mut app, window);
        let renders = renders.clone();
        let open_for_render = open.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-modal-view-cache-reuse",
            move |cx| {
                vec![
                    cx.view_cache(fret_ui::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.keyed("owner", |cx| {
                            crate::OverlayController::request(
                                cx,
                                crate::OverlayRequest::modal(
                                    modal_id,
                                    None,
                                    open_for_render.clone(),
                                    crate::OverlayPresence::instant(true),
                                    Vec::new(),
                                ),
                            );
                            cx.text("owner")
                        })]
                    }),
                ]
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        modal_layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .modals
                .get(&(window, modal_id))
                .map(|entry| entry.layer)
                .or(modal_layer)
        });
        let layer = modal_layer.expect("modal layer");
        assert!(
            ui.is_layer_visible(layer),
            "owned cached modal should remain visible across cache-hit frames"
        );
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "cache-hit frames should reuse the producer subtree without rerendering it"
    );
}

#[test]
fn owned_cached_popover_request_is_pruned_when_request_owner_is_removed() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let popover_id = GlobalElementId(0x44);
    let mut show_owner = true;
    let mut popover_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..2 {
        begin_frame(&mut app, window);
        let mut trigger_id: Option<GlobalElementId> = None;
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-popover-prune",
            |cx| {
                let mut out = vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_id = Some(id);
                        Vec::new()
                    },
                )];

                if show_owner {
                    let trigger = trigger_id.expect("trigger id");
                    out.push(cx.keyed("owner", |cx| {
                        crate::OverlayController::request(
                            cx,
                            crate::OverlayRequest::dismissible_popover(
                                popover_id,
                                trigger,
                                open.clone(),
                                crate::OverlayPresence::instant(true),
                                Vec::new(),
                            ),
                        );
                        cx.text("owner")
                    }));
                }

                out
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        if show_owner {
            popover_layer =
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    overlays
                        .popovers
                        .get(&(window, popover_id))
                        .map(|entry| entry.layer)
                });
            let layer = popover_layer.expect("popover layer");
            assert!(ui.is_layer_visible(layer));
            show_owner = false;
        }
    }

    let layer = popover_layer.expect("popover layer");
    assert!(
        ui.layer_root(layer).is_none(),
        "owned popover layer should be removed once its request owner disappears"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays.popovers.contains_key(&(window, popover_id)),
            "active popover entry should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_popover_requests
                .contains_key(&(window, popover_id)),
            "cached popover declaration should be pruned with the removed owner"
        );
    });
}

#[test]
fn owned_cached_popover_request_stays_visible_during_view_cache_reuse() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let popover_id = GlobalElementId(0x45);
    let renders = Arc::new(AtomicUsize::new(0));
    let mut popover_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..4 {
        begin_frame(&mut app, window);
        let renders = renders.clone();
        let open_for_render = open.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-popover-view-cache-reuse",
            move |cx| {
                vec![
                    cx.view_cache(fret_ui::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        let mut trigger_id: Option<GlobalElementId> = None;
                        let mut out = vec![cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(80.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx, _st, id| {
                                trigger_id = Some(id);
                                Vec::new()
                            },
                        )];
                        let trigger = trigger_id.expect("trigger id");
                        out.push(cx.keyed("owner", |cx| {
                            crate::OverlayController::request(
                                cx,
                                crate::OverlayRequest::dismissible_popover(
                                    popover_id,
                                    trigger,
                                    open_for_render.clone(),
                                    crate::OverlayPresence::instant(true),
                                    Vec::new(),
                                ),
                            );
                            cx.text("owner")
                        }));
                        out
                    }),
                ]
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        popover_layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .popovers
                .get(&(window, popover_id))
                .map(|entry| entry.layer)
                .or(popover_layer)
        });
        let layer = popover_layer.expect("popover layer");
        assert!(
            ui.is_layer_visible(layer),
            "owned cached popover should remain visible across cache-hit frames"
        );
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "cache-hit frames should reuse the popover producer subtree without rerendering it"
    );
}

#[test]
fn owned_cached_tooltip_request_is_pruned_immediately_when_request_owner_is_removed() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let tooltip_id = GlobalElementId(0x46);
    let mut show_owner = true;
    let mut tooltip_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..2 {
        begin_frame(&mut app, window);
        let mut trigger_id: Option<GlobalElementId> = None;
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-tooltip-prune",
            |cx| {
                let mut out = vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_id = Some(id);
                        Vec::new()
                    },
                )];

                if show_owner {
                    let trigger = trigger_id.expect("trigger id");
                    out.push(cx.keyed("owner", |cx| {
                        let mut request = crate::OverlayRequest::tooltip(
                            tooltip_id,
                            open.clone(),
                            crate::OverlayPresence::instant(true),
                            Vec::new(),
                        );
                        request.trigger = Some(trigger);
                        crate::OverlayController::request(cx, request);
                        cx.text("owner")
                    }));
                }

                out
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        if show_owner {
            tooltip_layer =
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    overlays
                        .tooltips
                        .get(&(window, tooltip_id))
                        .map(|entry| entry.layer)
                });
            let layer = tooltip_layer.expect("tooltip layer");
            assert!(ui.is_layer_visible(layer));
            show_owner = false;
        }
    }

    let layer = tooltip_layer.expect("tooltip layer");
    assert!(
        ui.layer_root(layer).is_none(),
        "owned tooltip layer should be removed immediately once its request owner disappears"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays.tooltips.contains_key(&(window, tooltip_id)),
            "active tooltip entry should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_tooltip_requests
                .contains_key(&(window, tooltip_id)),
            "cached tooltip declaration should be pruned with the removed owner"
        );
    });
}

#[test]
fn owned_cached_tooltip_request_stays_visible_during_view_cache_reuse() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let open = app.models_mut().insert(true);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let tooltip_id = GlobalElementId(0x47);
    let renders = Arc::new(AtomicUsize::new(0));
    let mut tooltip_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..4 {
        begin_frame(&mut app, window);
        let renders = renders.clone();
        let open_for_render = open.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-tooltip-view-cache-reuse",
            move |cx| {
                vec![
                    cx.view_cache(fret_ui::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        let mut trigger_id: Option<GlobalElementId> = None;
                        let mut out = vec![cx.pressable_with_id(
                            PressableProps {
                                layout: {
                                    let mut layout = LayoutStyle::default();
                                    layout.size.width = Length::Px(Px(80.0));
                                    layout.size.height = Length::Px(Px(32.0));
                                    layout
                                },
                                ..Default::default()
                            },
                            |_cx, _st, id| {
                                trigger_id = Some(id);
                                Vec::new()
                            },
                        )];
                        let trigger = trigger_id.expect("trigger id");
                        out.push(cx.keyed("owner", |cx| {
                            let mut request = crate::OverlayRequest::tooltip(
                                tooltip_id,
                                open_for_render.clone(),
                                crate::OverlayPresence::instant(true),
                                Vec::new(),
                            );
                            request.trigger = Some(trigger);
                            crate::OverlayController::request(cx, request);
                            cx.text("owner")
                        }));
                        out
                    }),
                ]
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        tooltip_layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .tooltips
                .get(&(window, tooltip_id))
                .map(|entry| entry.layer)
                .or(tooltip_layer)
        });
        let layer = tooltip_layer.expect("tooltip layer");
        assert!(
            ui.is_layer_visible(layer),
            "owned cached tooltip should remain visible across cache-hit frames"
        );
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "cache-hit frames should reuse the tooltip producer subtree without rerendering it"
    );
}

#[test]
fn owned_cached_toast_layer_request_is_pruned_when_request_owner_is_removed() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);

    let store = crate::OverlayController::toast_store(&mut app);
    let _ = crate::OverlayController::toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello"),
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let toast_layer_id = GlobalElementId(0x48);
    let mut show_owner = true;
    let mut toast_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..2 {
        begin_frame(&mut app, window);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-toast-layer-prune",
            |cx| {
                let mut out = Vec::new();
                if show_owner {
                    out.push(cx.keyed("owner", |cx| {
                        crate::OverlayController::request(
                            cx,
                            crate::OverlayRequest::toast_layer(toast_layer_id, store.clone())
                                .toast_style(zero_motion_toast_style()),
                        );
                        cx.text("owner")
                    }));
                }
                out
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        if show_owner {
            toast_layer =
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    overlays
                        .toast_layers
                        .get(&(window, toast_layer_id))
                        .map(|entry| entry.layer)
                });
            let layer = toast_layer.expect("toast layer");
            assert!(ui.is_layer_visible(layer));
            show_owner = false;
        }
    }

    let layer = toast_layer.expect("toast layer");
    assert!(
        ui.layer_root(layer).is_none(),
        "owned toast layer should be removed once its request owner disappears"
    );
    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        assert!(
            !overlays
                .toast_layers
                .contains_key(&(window, toast_layer_id)),
            "active toast layer entry should be pruned with the removed owner"
        );
        assert!(
            !overlays
                .cached_toast_layer_requests
                .contains_key(&(window, toast_layer_id)),
            "cached toast layer declaration should be pruned with the removed owner"
        );
    });
}

#[test]
fn owned_cached_toast_layer_request_stays_visible_during_view_cache_reuse() {
    let mut app = App::new();
    let mut ui = UiTree::new();
    let mut services = FakeServices::default();
    let window = AppWindowId::default();
    ui.set_window(window);
    ui.set_view_cache_enabled(true);

    let store = crate::OverlayController::toast_store(&mut app);
    let _ = crate::OverlayController::toast_action(
        &mut UiActionHostAdapter { app: &mut app },
        store.clone(),
        window,
        ToastRequest::new("Hello"),
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        fret_core::Size::new(Px(200.0), Px(120.0)),
    );

    let toast_layer_id = GlobalElementId(0x49);
    let renders = Arc::new(AtomicUsize::new(0));
    let mut toast_layer: Option<fret_ui::tree::UiLayerId> = None;

    for _frame in 0..4 {
        begin_frame(&mut app, window);
        let renders = renders.clone();
        let store_for_render = store.clone();
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "owned-cached-toast-layer-view-cache-reuse",
            move |cx| {
                vec![
                    cx.view_cache(fret_ui::element::ViewCacheProps::default(), |cx| {
                        renders.fetch_add(1, Ordering::SeqCst);
                        vec![cx.keyed("owner", |cx| {
                            crate::OverlayController::request(
                                cx,
                                crate::OverlayRequest::toast_layer(
                                    toast_layer_id,
                                    store_for_render.clone(),
                                )
                                .toast_style(zero_motion_toast_style()),
                            );
                            cx.text("owner")
                        })]
                    }),
                ]
            },
        );
        ui.set_root(root);
        crate::OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        toast_layer = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .toast_layers
                .get(&(window, toast_layer_id))
                .map(|entry| entry.layer)
                .or(toast_layer)
        });
        let layer = toast_layer.expect("toast layer");
        assert!(
            ui.is_layer_visible(layer),
            "owned cached toast layer should remain visible across cache-hit frames"
        );
    }

    assert_eq!(
        renders.load(Ordering::SeqCst),
        1,
        "cache-hit frames should reuse the toast-layer producer subtree without rerendering it"
    );
}
