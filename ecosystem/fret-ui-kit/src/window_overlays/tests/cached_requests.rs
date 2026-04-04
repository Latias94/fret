use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
            overlays.modals.get(&(window, modal_id)).is_none(),
            "active modal entry should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_modal_requests
                .get(&(window, modal_id))
                .is_none(),
            "cached modal declaration should be pruned with the removed owner"
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
            overlays.hover_overlays.get(&(window, hover_id)).is_none(),
            "active hover entry should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_hover_overlay_requests
                .get(&(window, hover_id))
                .is_none(),
            "cached hover declaration should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_hover_overlay_pointer_move_handlers
                .get(&(window, hover_id))
                .is_none(),
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
            overlays.popovers.get(&(window, popover_id)).is_none(),
            "active popover entry should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_popover_requests
                .get(&(window, popover_id))
                .is_none(),
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
            overlays.tooltips.get(&(window, tooltip_id)).is_none(),
            "active tooltip entry should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_tooltip_requests
                .get(&(window, tooltip_id))
                .is_none(),
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
            overlays
                .toast_layers
                .get(&(window, toast_layer_id))
                .is_none(),
            "active toast layer entry should be pruned with the removed owner"
        );
        assert!(
            overlays
                .cached_toast_layer_requests
                .get(&(window, toast_layer_id))
                .is_none(),
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
