use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{AppWindowId, Color, NodeId, Point, Px, Rect, Transform2D, WindowMetricsService};
use fret_runtime::DRAG_KIND_DOCK_PANEL;
use fret_ui::action::{
    ActionCx, AutoFocusRequestCx, DismissReason, DismissRequestCx, OnCloseAutoFocus, UiActionHost,
    UiActionHostAdapter, UiActionHostExt, UiFocusActionHost,
};
use fret_ui::declarative;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{Invalidation, UiHost, UiTree};

use crate::primitives::dismissable_layer as dismissable_layer_prim;
use crate::primitives::focus_scope as focus_scope_prim;

use super::state::{
    ActiveHoverOverlay, ActiveModal, ActivePopover, ActiveToastLayer, ActiveTooltip,
    NonModalDismissibleLayerPolicy, OVERLAY_CACHE_TTL_FRAMES, OverlayLayer, WindowOverlays,
    apply_hover_layer, apply_modal_layer, apply_non_modal_dismissible_layer, apply_tooltip_layer,
};
use super::toast::{ToastEntry, ToastTimerOutcome};
use super::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, ToastPosition,
    ToastVariant, TooltipRequest, dismiss_toast_action,
};

#[derive(Default)]
struct ToastHoverPauseState {
    hovered: bool,
}

struct OverlayFocusActionHostAdapter<'a, H: UiHost> {
    app: &'a mut H,
    ui: &'a mut UiTree<H>,
    window: AppWindowId,
}

impl<'a, H: UiHost> UiActionHost for OverlayFocusActionHostAdapter<'a, H> {
    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
        self.app.models_mut()
    }

    fn push_effect(&mut self, effect: fret_runtime::Effect) {
        self.app.push_effect(effect);
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        self.app.request_redraw(window);
    }

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        self.app.next_timer_token()
    }
}

impl<'a, H: UiHost> UiFocusActionHost for OverlayFocusActionHostAdapter<'a, H> {
    fn request_focus(&mut self, target: GlobalElementId) {
        if let Some(node) = fret_ui::elements::node_for_element(self.app, self.window, target) {
            self.ui.set_focus(Some(node));
        }
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn toast_icon_glyph(variant: ToastVariant) -> Option<&'static str> {
    match variant {
        ToastVariant::Success => Some("\u{2713}"),
        ToastVariant::Info => Some("i"),
        ToastVariant::Warning => Some("!"),
        ToastVariant::Error | ToastVariant::Destructive => Some("\u{00D7}"),
        ToastVariant::Loading => None,
        ToastVariant::Default => None,
    }
}

fn capture_conflicts_with_layer(
    arbitration: fret_ui::tree::UiInputArbitrationSnapshot,
    layer: UiLayerId,
) -> bool {
    arbitration.pointer_capture_active
        && (arbitration.pointer_capture_multiple_layers
            || arbitration.pointer_capture_layer != Some(layer))
}

fn should_suspend_pointer_gating_for_capture(
    open: bool,
    capture_conflicts_with_layer: bool,
    disable_outside_pointer_events: bool,
    consume_outside_pointer_events: bool,
) -> bool {
    open && capture_conflicts_with_layer
        && (disable_outside_pointer_events || consume_outside_pointer_events)
}

pub fn render<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let frame_id = app.frame_id();
    let dock_drag_affects_window = app.any_drag_session(|d| {
        d.kind == DRAG_KIND_DOCK_PANEL && (d.source_window == window || d.current_window == window)
    });
    let arbitration = ui.input_arbitration_snapshot();

    let focused_now = app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.focused(window));
    let scale_factor_now = app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(window));

    let focus_now = ui.focus();
    let (focus_lost, resized, dock_drag_restore_focus) =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let w = overlays.windows.entry(window).or_default();

            let bounds_changed = w.last_bounds.is_some_and(|last| last.size != bounds.size);
            let scale_changed = w
                .last_scale_factor
                .is_some_and(|last| Some(last) != scale_factor_now);
            let resized = bounds_changed || scale_changed;

            let focus_lost = matches!((w.last_focused, focused_now), (Some(true), Some(false)));

            w.last_bounds = Some(bounds);
            if let Some(focused_now) = focused_now {
                w.last_focused = Some(focused_now);
            }
            if let Some(scale_factor_now) = scale_factor_now {
                w.last_scale_factor = Some(scale_factor_now);
            }

            let started = dock_drag_affects_window && !w.dock_drag_active_last;
            let ended = !dock_drag_affects_window && w.dock_drag_active_last;
            w.dock_drag_active_last = dock_drag_affects_window;
            if started {
                w.dock_drag_restore_focus = focus_now;
            }
            let restore = ended.then(|| w.dock_drag_restore_focus.take()).flatten();

            (focus_lost, resized, restore)
        });

    if let Some(restore) = dock_drag_restore_focus
        && ui.focus().is_none()
        && ui.node_layer(restore).is_some()
    {
        ui.set_focus(Some(restore));
    }

    let (
        mut modal_requests,
        mut popover_requests,
        mut hover_overlay_requests,
        mut tooltip_requests,
        mut toast_requests,
    ) = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .windows
            .get_mut(&window)
            .map(|w| {
                (
                    std::mem::take(&mut w.modals),
                    std::mem::take(&mut w.popovers),
                    std::mem::take(&mut w.hover_overlays),
                    std::mem::take(&mut w.tooltips),
                    std::mem::take(&mut w.toasts),
                )
            })
            .unwrap_or_default()
    });

    // When view caching skips rerendering the subtree that emits overlay requests, the per-frame
    // request lists will be empty (or missing specific overlays). Keep a cached "declaration"
    // and synthesize requests for overlays that are currently open so overlay behavior remains
    // correct under view caching.
    //
    // Notes:
    // - This intentionally treats close transitions as "instant" when the request producer is
    //   not rerendering: if `open` flips false, the overlay disappears as soon as we stop
    //   synthesizing a request.
    // - Without this, scripts that rely on Radix-style overlay semantics can fail when view
    //   caching is enabled (the overlay request vanishes for a frame and the overlay unmounts).
    let modal_request_ids: HashSet<GlobalElementId> = modal_requests.iter().map(|r| r.id).collect();
    let popover_request_ids: HashSet<GlobalElementId> =
        popover_requests.iter().map(|r| r.id).collect();
    let hover_request_ids: HashSet<GlobalElementId> =
        hover_overlay_requests.iter().map(|r| r.id).collect();
    let tooltip_request_ids: HashSet<GlobalElementId> =
        tooltip_requests.iter().map(|r| r.id).collect();
    let toast_request_ids: HashSet<GlobalElementId> = toast_requests.iter().map(|r| r.id).collect();

    let (extra_modals, extra_popovers, extra_hover_overlays, extra_tooltips, extra_toasts) = app
        .with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
            let mut modals: Vec<ModalRequest> = Vec::new();
            let mut popovers: Vec<DismissiblePopoverRequest> = Vec::new();
            let mut hover_overlays: Vec<HoverOverlayRequest> = Vec::new();
            let mut tooltips: Vec<TooltipRequest> = Vec::new();
            let mut toasts: Vec<ToastLayerRequest> = Vec::new();

            for ((w, id), req) in overlays.cached_modal_requests.iter() {
                if *w != window || modal_request_ids.contains(id) {
                    continue;
                }
                let open_now = app.models().get_copied(&req.open).unwrap_or(false);
                if !open_now {
                    continue;
                }
                let mut req = req.clone();
                req.present = true;
                modals.push(req);
            }

            for ((w, id), req) in overlays.cached_popover_requests.iter() {
                if *w != window || popover_request_ids.contains(id) {
                    continue;
                }
                let open_now = app.models().get_copied(&req.open).unwrap_or(false);
                if !open_now {
                    continue;
                }
                let mut req = req.clone();
                req.present = true;
                popovers.push(req);
            }

            // Hover overlays and tooltips participate in view-caching synthesis, but with a short
            // TTL so stale cached requests cannot keep "ephemeral" overlays alive indefinitely.
            for ((w, id), req) in overlays.cached_hover_overlay_requests.iter() {
                if *w != window || hover_request_ids.contains(id) {
                    continue;
                }
                let Some(active) = overlays.hover_overlays.get(&(window, *id)) else {
                    continue;
                };
                if frame_id.0.saturating_sub(active.last_seen_frame.0) > OVERLAY_CACHE_TTL_FRAMES {
                    continue;
                }
                let open_now = app.models().get_copied(&req.open).unwrap_or(false);
                if !open_now {
                    continue;
                }
                let mut req = req.clone();
                req.present = true;
                hover_overlays.push(req);
            }

            for ((w, id), req) in overlays.cached_tooltip_requests.iter() {
                if *w != window || tooltip_request_ids.contains(id) {
                    continue;
                }
                let Some(active) = overlays.tooltips.get(&(window, *id)) else {
                    continue;
                };
                if frame_id.0.saturating_sub(active.last_seen_frame.0) > OVERLAY_CACHE_TTL_FRAMES {
                    continue;
                }
                let open_now = app.models().get_copied(&req.open).unwrap_or(false);
                if !open_now {
                    continue;
                }
                let mut req = req.clone();
                req.present = true;
                tooltips.push(req);
            }

            for ((w, id), req) in overlays.cached_toast_layer_requests.iter() {
                if *w != window || toast_request_ids.contains(id) {
                    continue;
                }
                toasts.push(req.clone());
            }

            (modals, popovers, hover_overlays, tooltips, toasts)
        });

    modal_requests.extend(extra_modals);
    popover_requests.extend(extra_popovers);
    hover_overlay_requests.extend(extra_hover_overlays);
    tooltip_requests.extend(extra_tooltips);
    toast_requests.extend(extra_toasts);

    let mut seen_modals: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_popovers: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_hover_overlays: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_tooltips: HashSet<GlobalElementId> = HashSet::new();
    let mut seen_toast_layers: HashSet<GlobalElementId> = HashSet::new();

    for req in modal_requests {
        if !req.present {
            continue;
        }
        seen_modals.insert(req.id);

        let mut open_now = app.models().get_copied(&req.open).unwrap_or(false);
        if open_now
            && ((focus_lost && req.close_on_window_focus_lost)
                || (resized && req.close_on_window_resize))
        {
            let _ = app.models_mut().update(&req.open, |v| *v = false);
            open_now = false;
        }

        let modal_id = req.id;
        let root_name = req.root_name.clone();
        let trigger = req.trigger;
        let initial_focus = req.initial_focus;
        let open = req.open;
        let on_open_auto_focus = req.on_open_auto_focus.clone();
        let on_close_auto_focus = req.on_close_auto_focus.clone();
        let on_dismiss_request = req.on_dismiss_request.clone();
        let children = req.children;

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            move |cx| {
                cx.dismissible_on_dismiss_request(on_dismiss_request.unwrap_or_else(|| {
                    Arc::new(
                        move |host: &mut dyn UiActionHost,
                              _cx: ActionCx,
                              _req: &mut DismissRequestCx| {
                            let _ = host.models_mut().update(&open, |v| *v = false);
                        },
                    )
                }));
                children
            },
        );

        let key = (window, modal_id);
        let restore_focus = ui.focus();

        let mut layer: Option<UiLayerId> = None;
        let mut created = false;
        let prev_open = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let entry = overlays.modals.entry(key).or_insert_with(|| {
                created = true;
                ActiveModal {
                    layer: ui.push_overlay_root_ex(root, true, true),
                    root_name: root_name.clone(),
                    trigger,
                    initial_focus,
                    open: false,
                    restore_focus: None,
                    close_auto_focus_handled: false,
                    close_auto_focus_prevented: false,
                    pending_initial_focus: false,
                }
            });
            let prev_open = entry.open;
            entry.root_name = root_name.clone();
            entry.trigger = trigger;
            entry.initial_focus = initial_focus;
            layer = Some(entry.layer);

            apply_modal_layer(ui, entry.layer, true);
            entry.open = open_now;
            if open_now {
                entry.close_auto_focus_handled = false;
                entry.close_auto_focus_prevented = false;
            }
            prev_open
        });

        let opening = open_now && (!prev_open || created);
        let closing = prev_open && !open_now;

        let mut open_auto_focus_prevented = false;
        if opening {
            if let Some(on_open_auto_focus) = on_open_auto_focus.as_ref() {
                let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                let mut req_cx = AutoFocusRequestCx::new();
                on_open_auto_focus(
                    &mut host,
                    ActionCx {
                        window,
                        target: trigger.unwrap_or(modal_id),
                    },
                    &mut req_cx,
                );
                open_auto_focus_prevented = req_cx.default_prevented();
            }

            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                if let Some(entry) = overlays.modals.get_mut(&key) {
                    entry.restore_focus = restore_focus;
                    entry.pending_initial_focus = !open_auto_focus_prevented;
                }
            });
        }

        if closing {
            let mut close_auto_focus_prevented = false;
            if let Some(on_close_auto_focus) = on_close_auto_focus.as_ref() {
                let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                let mut req_cx = AutoFocusRequestCx::new();
                on_close_auto_focus(
                    &mut host,
                    ActionCx {
                        window,
                        target: trigger.unwrap_or(modal_id),
                    },
                    &mut req_cx,
                );
                close_auto_focus_prevented = req_cx.default_prevented();
            }
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                if let Some(entry) = overlays.modals.get_mut(&key) {
                    entry.close_auto_focus_handled = true;
                    entry.close_auto_focus_prevented = close_auto_focus_prevented;
                }
            });

            let focus_now = ui.focus();
            let focus_in_layer = layer
                .is_some_and(|layer| focus_now.is_some_and(|n| ui.node_layer(n) == Some(layer)));
            if (focus_now.is_none() || focus_in_layer) && !close_auto_focus_prevented {
                let (trigger, restore_focus) =
                    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                        overlays
                            .modals
                            .get(&key)
                            .map(|entry| (entry.trigger, entry.restore_focus))
                            .unwrap_or((trigger, None))
                    });
                if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                    ui,
                    app,
                    window,
                    trigger,
                    restore_focus,
                ) {
                    ui.set_focus(Some(node));
                }
            }
        }

        let focus_in_layer = layer.is_some_and(|layer| {
            ui.focus()
                .is_some_and(|n| ui.node_layer(n).is_some_and(|lid| lid == layer))
        });
        let enforce_focus_containment =
            open_now && !focus_in_layer && !(opening && open_auto_focus_prevented);

        let pending_initial_focus =
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays
                    .modals
                    .get(&key)
                    .is_some_and(|entry| entry.pending_initial_focus)
            });

        if (opening && !open_auto_focus_prevented)
            || pending_initial_focus
            || enforce_focus_containment
        {
            let focus = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays.modals.get(&key).and_then(|p| p.initial_focus)
            });
            let applied =
                focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus);
            if !applied && enforce_focus_containment {
                ui.set_focus(Some(root));
            }
            if applied {
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    if let Some(entry) = overlays.modals.get_mut(&key) {
                        entry.pending_initial_focus = false;
                    }
                });
            }
        }
    }

    let modal_barrier_active = !seen_modals.is_empty();
    let modal_branch_nodes: Vec<NodeId> = if modal_barrier_active {
        let modal_layers: Vec<fret_ui::tree::UiLayerId> =
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays
                    .modals
                    .iter()
                    .filter(|((w, id), _)| *w == window && seen_modals.contains(id))
                    .map(|(_, entry)| entry.layer)
                    .collect()
            });
        modal_layers
            .into_iter()
            .filter_map(|layer| ui.layer_root(layer))
            .collect()
    } else {
        Vec::new()
    };

    for req in popover_requests {
        if dock_drag_affects_window {
            if req.present {
                let _ = app.models_mut().update(&req.open, |v| *v = false);
            }
            continue;
        }

        if !req.present {
            continue;
        }
        seen_popovers.insert(req.id);

        let focus_now = ui.focus();
        let disable_outside_pointer_events = req.disable_outside_pointer_events;

        // Click-through overlays (popover-like) treat the trigger as an implicit branch so a
        // trigger click doesn't first dismiss the overlay and then immediately re-open it.
        //
        // Menu-like overlays that disable outside pointer interactions should *not* treat the
        // trigger as a branch: the trigger press must be considered "outside" so it can close the
        // overlay without activating the underlay.
        let mut dismissable_branch_nodes = if disable_outside_pointer_events {
            dismissable_layer_prim::resolve_branch_nodes_for_elements(
                app,
                window,
                &req.dismissable_branches,
            )
        } else {
            dismissable_layer_prim::resolve_branch_nodes_for_trigger_and_elements(
                app,
                window,
                req.trigger,
                &req.dismissable_branches,
            )
        };
        dismissable_branch_nodes.extend(modal_branch_nodes.iter().copied());

        let mut open_now = app.models().get_copied(&req.open).unwrap_or(false);
        if open_now
            && ((focus_lost && req.close_on_window_focus_lost)
                || (resized && req.close_on_window_resize))
        {
            let _ = app.models_mut().update(&req.open, |v| *v = false);
            open_now = false;
        }

        let popover_id = req.id;
        let root_name = req.root_name.clone();
        let trigger = req.trigger;
        let initial_focus = req.initial_focus;
        let consume_outside_pointer_events = req.consume_outside_pointer_events;
        let wants_pointer_move_events = req.on_pointer_move.is_some();
        let open = req.open;
        let open_for_dismiss = open.clone();
        let on_pointer_move = req.on_pointer_move.clone();
        let on_dismiss_request = req.on_dismiss_request.clone();
        let on_open_auto_focus = req.on_open_auto_focus.clone();
        let on_close_auto_focus = req.on_close_auto_focus.clone();
        let on_dismiss_request_for_root = on_dismiss_request.clone();
        let children = req.children;

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &root_name,
            move |cx| {
                if let Some(on_pointer_move) = on_pointer_move {
                    cx.dismissible_on_pointer_move(on_pointer_move);
                }
                let on_dismiss_request = on_dismiss_request_for_root.clone();
                cx.dismissible_on_dismiss_request(on_dismiss_request.unwrap_or_else(|| {
                    Arc::new(
                        move |host: &mut dyn UiActionHost,
                              _cx: ActionCx,
                              _req: &mut DismissRequestCx| {
                            let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                        },
                    )
                }));
                children
            },
        );

        let key = (window, popover_id);
        let restore_focus = ui.focus();

        let mut pending_initial_focus = false;
        let mut created = false;
        let mut prev_open = false;
        let mut layer: Option<UiLayerId> = None;
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
            let entry = overlays.popovers.entry(key).or_insert_with(|| {
                created = true;
                ActivePopover {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: root_name.clone(),
                    trigger,
                    initial_focus,
                    pending_initial_focus: false,
                    consume_outside_pointer_events,
                    disable_outside_pointer_events,
                    close_auto_focus_handled: false,
                    close_auto_focus_prevented: false,
                    open: false,
                    restore_focus: None,
                    last_focus: focus_now,
                }
            });
            prev_open = entry.open;
            entry.root_name = root_name.clone();
            entry.trigger = trigger;
            entry.initial_focus = initial_focus;
            entry.consume_outside_pointer_events = consume_outside_pointer_events;
            entry.disable_outside_pointer_events = disable_outside_pointer_events;
            layer = Some(entry.layer);

            // Input arbitration: avoid introducing pointer occlusion mid-capture.
            //
            // Menu-like overlays (Radix `disableOutsidePointerEvents`) normally want to occlude
            // underlay pointer input while open. If another layer is currently capturing the
            // pointer (viewport drags, resizers, etc.), enabling occlusion can change routing
            // semantics in surprising ways.
            //
            // Do not force-close the overlay here: open state is component-owned and closing as a
            // side effect of input arbitration produces flicker and breaks pointer-open focus.
            // Instead, temporarily suspend pointer gating until capture is released.
            let capture_conflicts_with_layer =
                capture_conflicts_with_layer(arbitration, entry.layer);
            let suspend_pointer_gating_for_capture = should_suspend_pointer_gating_for_capture(
                open_now,
                capture_conflicts_with_layer,
                disable_outside_pointer_events,
                consume_outside_pointer_events,
            );
            let effective_interactive = open_now && !suspend_pointer_gating_for_capture;

            if open_now
                && let Some(layer_root) = ui.layer_root(entry.layer)
                && dismissable_layer_prim::should_dismiss_on_focus_outside(
                    ui,
                    layer_root,
                    focus_now,
                    entry.last_focus,
                    &dismissable_branch_nodes,
                )
            {
                if let Some(on_dismiss_request) = on_dismiss_request.as_ref() {
                    let mut host = UiActionHostAdapter { app };
                    let mut req_cx = DismissRequestCx::new(DismissReason::FocusOutside);
                    on_dismiss_request(
                        &mut host,
                        ActionCx {
                            window,
                            target: trigger,
                        },
                        &mut req_cx,
                    );

                    if !req_cx.default_prevented() {
                        let _ = app.models_mut().update(&open, |v| *v = false);
                        open_now = false;
                    }
                    open_now = app
                        .models_mut()
                        .read(&open, |v| *v)
                        .ok()
                        .unwrap_or(open_now);
                } else {
                    let _ = app.models_mut().update(&open, |v| *v = false);
                    open_now = false;
                }
            }

            let dismissable_branches = if open_now {
                dismissable_branch_nodes.clone()
            } else {
                Vec::new()
            };
            apply_non_modal_dismissible_layer(
                ui,
                entry.layer,
                true,
                effective_interactive,
                NonModalDismissibleLayerPolicy {
                    dismissable_branches,
                    consume_outside_pointer_events,
                    disable_outside_pointer_events,
                    wants_pointer_move_events,
                },
            );

            entry.open = open_now;
            if open_now {
                entry.close_auto_focus_handled = false;
                entry.close_auto_focus_prevented = false;
            }
            entry.last_focus = focus_now;
            pending_initial_focus = entry.pending_initial_focus;
        });

        let opening = open_now && (!prev_open || created);
        let closing = prev_open && !open_now;

        let mut open_auto_focus_prevented = false;
        if opening {
            if let Some(on_open_auto_focus) = on_open_auto_focus.as_ref() {
                let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                let mut req_cx = AutoFocusRequestCx::new();
                on_open_auto_focus(
                    &mut host,
                    ActionCx {
                        window,
                        target: trigger,
                    },
                    &mut req_cx,
                );
                open_auto_focus_prevented = req_cx.default_prevented();
            }

            app.with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
                if let Some(entry) = overlays.popovers.get_mut(&key) {
                    entry.restore_focus = restore_focus
                        .or_else(|| fret_ui::elements::node_for_element(app, window, trigger));
                    entry.pending_initial_focus = !open_auto_focus_prevented;
                }
            });
        }

        if closing
            && (consume_outside_pointer_events
                || layer.is_some_and(|layer| {
                    focus_scope_prim::should_restore_focus_for_non_modal_overlay(ui, layer)
                }))
        {
            let focus_now = ui.focus();
            let focus_in_layer = layer
                .is_some_and(|layer| focus_now.is_some_and(|n| ui.node_layer(n) == Some(layer)));
            let focus_cleared_by_modal_scope = modal_barrier_active && focus_now.is_none();

            let mut close_auto_focus_prevented = false;
            if !focus_cleared_by_modal_scope {
                if let Some(on_close_auto_focus) = on_close_auto_focus.as_ref() {
                    let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                    let mut req_cx = AutoFocusRequestCx::new();
                    on_close_auto_focus(
                        &mut host,
                        ActionCx {
                            window,
                            target: trigger,
                        },
                        &mut req_cx,
                    );
                    close_auto_focus_prevented = req_cx.default_prevented();
                }
            }
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                if let Some(entry) = overlays.popovers.get_mut(&key) {
                    entry.close_auto_focus_handled = true;
                    entry.close_auto_focus_prevented = close_auto_focus_prevented;
                }
            });

            if (!close_auto_focus_prevented)
                && ((!focus_cleared_by_modal_scope && focus_now.is_none()) || focus_in_layer)
            {
                let restore_focus =
                    app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                        overlays
                            .popovers
                            .get(&key)
                            .and_then(|entry| entry.restore_focus)
                    });
                if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                    ui,
                    app,
                    window,
                    Some(trigger),
                    restore_focus,
                ) {
                    ui.set_focus(Some(node));
                }
            }
        }

        let pending_initial_focus =
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays
                    .popovers
                    .get(&key)
                    .is_some_and(|entry| entry.pending_initial_focus)
            });

        let should_focus_initial = opening && !open_auto_focus_prevented;

        if should_focus_initial || pending_initial_focus {
            if should_focus_initial && open_now && consume_outside_pointer_events {
                ui.set_focus(Some(root));
            }
            let focus = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.get(&key).and_then(|p| p.initial_focus)
            });
            let focus_before = ui.focus();
            let applied =
                focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus);
            if applied {
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    if let Some(entry) = overlays.popovers.get_mut(&key) {
                        entry.pending_initial_focus = false;
                    }
                });
            }

            if should_focus_initial
                && open_now
                && consume_outside_pointer_events
                && (ui.focus() == focus_before || !applied)
            {
                // Menu-like overlays should move focus inside the overlay on pointer-open to
                // match Radix `onEntryFocus` outcomes (focus stays within the menu, but the
                // first item is not automatically focused).
                ui.set_focus(Some(root));
            }
        }
    }

    let to_hide_popovers: Vec<(
        UiLayerId,
        GlobalElementId,
        bool,
        Option<NodeId>,
        bool,
        bool,
        Option<OnCloseAutoFocus>,
    )> = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let mut out: Vec<(
            UiLayerId,
            GlobalElementId,
            bool,
            Option<NodeId>,
            bool,
            bool,
            Option<OnCloseAutoFocus>,
        )> = Vec::new();
        for ((w, id), active) in overlays.popovers.iter() {
            if *w != window || seen_popovers.contains(id) {
                continue;
            }
            let on_close_auto_focus = overlays
                .cached_popover_requests
                .get(&(*w, *id))
                .and_then(|req| req.on_close_auto_focus.clone());
            out.push((
                active.layer,
                active.trigger,
                active.consume_outside_pointer_events,
                active.restore_focus,
                active.close_auto_focus_handled,
                active.close_auto_focus_prevented,
                on_close_auto_focus,
            ));
        }
        out
    });

    let to_hide_modals: Vec<(
        UiLayerId,
        GlobalElementId,
        Option<GlobalElementId>,
        Option<NodeId>,
        bool,
        bool,
        Option<OnCloseAutoFocus>,
    )> = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        overlays
            .modals
            .iter()
            .filter_map(|((w, id), active)| {
                if *w != window || seen_modals.contains(id) {
                    return None;
                }
                let on_close_auto_focus = overlays
                    .cached_modal_requests
                    .get(&(*w, *id))
                    .and_then(|req| req.on_close_auto_focus.clone());
                Some((
                    active.layer,
                    *id,
                    active.trigger,
                    active.restore_focus,
                    active.close_auto_focus_handled,
                    active.close_auto_focus_prevented,
                    on_close_auto_focus,
                ))
            })
            .collect()
    });

    for (
        layer,
        trigger,
        consume_outside_pointer_events,
        restore_focus,
        close_auto_focus_handled,
        close_auto_focus_prevented,
        on_close_auto_focus,
    ) in to_hide_popovers
    {
        let mut close_auto_focus_prevented = close_auto_focus_prevented;
        if !close_auto_focus_handled {
            if let Some(on_close_auto_focus) = on_close_auto_focus.as_ref() {
                let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                let mut req_cx = AutoFocusRequestCx::new();
                on_close_auto_focus(
                    &mut host,
                    ActionCx {
                        window,
                        target: trigger,
                    },
                    &mut req_cx,
                );
                close_auto_focus_prevented = req_cx.default_prevented();
            }
        }

        let focus_now = ui.focus();
        let focus_in_layer = focus_now.is_some_and(|n| ui.node_layer(n) == Some(layer));
        let focus_cleared_by_modal_scope = modal_barrier_active && focus_now.is_none();

        // Radix-aligned outcome for menu-like overlays (ADR 0069):
        // when the overlay consumes outside pointer-down events (non-click-through), it's safe to
        // always restore focus to the trigger on unmount (like modals).
        if !close_auto_focus_prevented
            && (consume_outside_pointer_events
                || (focus_in_layer
                    || (!focus_cleared_by_modal_scope
                        && focus_scope_prim::should_restore_focus_for_non_modal_overlay(
                            ui, layer,
                        ))))
        {
            apply_non_modal_dismissible_layer(
                ui,
                layer,
                false,
                false,
                NonModalDismissibleLayerPolicy {
                    dismissable_branches: Vec::new(),
                    consume_outside_pointer_events: false,
                    disable_outside_pointer_events: false,
                    wants_pointer_move_events: false,
                },
            );
            if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                ui,
                app,
                window,
                Some(trigger),
                restore_focus,
            ) {
                ui.set_focus(Some(node));
            }
        } else {
            apply_non_modal_dismissible_layer(
                ui,
                layer,
                false,
                false,
                NonModalDismissibleLayerPolicy {
                    dismissable_branches: Vec::new(),
                    consume_outside_pointer_events: false,
                    disable_outside_pointer_events: false,
                    wants_pointer_move_events: false,
                },
            );
        }
    }

    for (
        layer,
        modal_id,
        trigger,
        restore_focus,
        close_auto_focus_handled,
        close_auto_focus_prevented,
        on_close_auto_focus,
    ) in to_hide_modals
    {
        let mut close_auto_focus_prevented = close_auto_focus_prevented;
        if !close_auto_focus_handled {
            if let Some(on_close_auto_focus) = on_close_auto_focus.as_ref() {
                let mut host = OverlayFocusActionHostAdapter { app, ui, window };
                let mut req_cx = AutoFocusRequestCx::new();
                on_close_auto_focus(
                    &mut host,
                    ActionCx {
                        window,
                        target: trigger.unwrap_or(modal_id),
                    },
                    &mut req_cx,
                );
                close_auto_focus_prevented = req_cx.default_prevented();
            }
        }

        // Modals should restore focus deterministically on close (Radix-style): underlay focus
        // changes cannot happen while the barrier is installed, so it's safe to always restore on
        // unmount.
        apply_modal_layer(ui, layer, false);

        if !close_auto_focus_prevented {
            if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                ui,
                app,
                window,
                trigger,
                restore_focus,
            ) {
                ui.set_focus(Some(node));
            }
        }
    }

    for req in hover_overlay_requests {
        if dock_drag_affects_window {
            continue;
        }

        if !req.present {
            continue;
        }
        let from_producer = hover_request_ids.contains(&req.id);
        seen_hover_overlays.insert(req.id);

        let open_now = app.models().get_copied(&req.open).unwrap_or(false);
        let interactive = req.interactive && open_now;
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |_cx| req.children,
        );

        let key = (window, req.id);
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .hover_overlays
                .entry(key)
                .or_insert_with(|| ActiveHoverOverlay {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    open: req.open.clone(),
                    last_seen_frame: frame_id,
                });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.open = req.open;
            if from_producer {
                entry.last_seen_frame = frame_id;
            }
            let present = !capture_conflicts_with_layer(arbitration, entry.layer);
            apply_hover_layer(ui, entry.layer, present, interactive && present);
        });
    }

    let to_hide_hover_overlays: Vec<(UiLayerId, GlobalElementId)> =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .hover_overlays
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_hover_overlays.contains(id) {
                        return None;
                    }
                    Some((active.layer, active.trigger))
                })
                .collect()
        });

    for (layer, trigger) in to_hide_hover_overlays {
        let focus = ui.focus();
        if focus.is_some_and(|n| ui.node_layer(n) == Some(layer))
            && let Some(trigger_node) = fret_ui::elements::node_for_element(app, window, trigger)
        {
            apply_hover_layer(ui, layer, false, false);
            ui.set_focus(Some(trigger_node));
        } else {
            apply_hover_layer(ui, layer, false, false);
        }
    }

    for req in tooltip_requests {
        if dock_drag_affects_window {
            continue;
        }

        if !req.present {
            continue;
        }
        let from_producer = tooltip_request_ids.contains(&req.id);
        seen_tooltips.insert(req.id);

        let open_now = app.models().get_copied(&req.open).unwrap_or(false);
        let interactive = req.interactive && open_now;
        let on_dismiss_request = req.on_dismiss_request.clone();
        let on_pointer_move = req.on_pointer_move.clone();
        let children = req.children;
        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            move |cx| {
                if let Some(on_dismiss_request) = on_dismiss_request {
                    cx.dismissible_on_dismiss_request(on_dismiss_request);
                }
                if let Some(on_pointer_move) = on_pointer_move {
                    cx.dismissible_on_pointer_move(on_pointer_move);
                }
                children
            },
        );

        let key = (window, req.id);
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .tooltips
                .entry(key)
                .or_insert_with(|| ActiveTooltip {
                    layer: ui.push_overlay_root_ex(root, false, false),
                    root_name: req.root_name.clone(),
                    open: req.open.clone(),
                    last_seen_frame: frame_id,
                });
            entry.root_name = req.root_name.clone();
            entry.open = req.open;
            if from_producer {
                entry.last_seen_frame = frame_id;
            }
            let present = !capture_conflicts_with_layer(arbitration, entry.layer);
            let interactive = interactive && present;

            apply_tooltip_layer(ui, entry.layer, present, interactive);

            let wants_outside_press_observer = req.on_dismiss_request.is_some();
            let wants_pointer_move_events = req.on_pointer_move.is_some();

            ui.set_layer_wants_pointer_down_outside_events(
                entry.layer,
                present && interactive && wants_outside_press_observer,
            );
            ui.set_layer_wants_pointer_move_events(
                entry.layer,
                present && interactive && wants_pointer_move_events,
            );

            if interactive {
                ui.set_layer_scroll_dismiss_elements(
                    entry.layer,
                    req.trigger.into_iter().collect(),
                );
            } else {
                ui.set_layer_scroll_dismiss_elements(entry.layer, Vec::new());
            }
        });
    }

    let to_hide_tooltips: Vec<UiLayerId> =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .tooltips
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_tooltips.contains(id) {
                        return None;
                    }
                    Some(active.layer)
                })
                .collect()
        });

    for layer in to_hide_tooltips {
        apply_tooltip_layer(ui, layer, false, false);
    }

    for req in toast_requests {
        seen_toast_layers.insert(req.id);

        let store = req.store;
        let store_for_render = store.clone();
        let position = req.position;
        let margin_override = req.margin;
        let gap_override = req.gap;
        let toast_min_width_override = req.toast_min_width;
        let toast_max_width_override = req.toast_max_width;
        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            move |cx| {
                cx.observe_model(&store_for_render, Invalidation::Paint);

                let hook_store = store_for_render.downgrade();
                cx.timer_on_timer_for(
                    cx.root_id(),
                    Arc::new(move |host, _cx, token| {
                        let remove_token = host.next_timer_token();
                        let outcome = host
                            .update_weak_model(&hook_store, |st| st.on_timer(token, remove_token));

                        let Some(outcome) = outcome else {
                            return false;
                        };

                        match outcome {
                            ToastTimerOutcome::Noop => false,
                            ToastTimerOutcome::RescheduleAuto {
                                window,
                                token,
                                after,
                            } => {
                                host.push_effect(fret_runtime::Effect::SetTimer {
                                    window: Some(window),
                                    token,
                                    after,
                                    repeat: None,
                                });
                                true
                            }
                            ToastTimerOutcome::BeganClose {
                                window,
                                remove_token,
                            } => {
                                host.push_effect(fret_runtime::Effect::SetTimer {
                                    window: Some(window),
                                    token: remove_token,
                                    after: super::toast::TOAST_CLOSE_DURATION,
                                    repeat: None,
                                });
                                host.request_redraw(window);
                                true
                            }
                            ToastTimerOutcome::Removed { window } => {
                                host.request_redraw(window);
                                true
                            }
                        }
                    }),
                );

                let toasts: Vec<ToastEntry> = cx
                    .app
                    .models()
                    .read(&store_for_render, |st| {
                        st.toasts_for_window(window).to_vec()
                    })
                    .unwrap_or_default();

                if toasts.is_empty() {
                    return Vec::new();
                }

                // Match common toast UX: top stacks show the newest item at the top edge, while
                // bottom stacks show the newest item at the bottom edge.
                let mut toasts = toasts;
                if matches!(
                    position,
                    ToastPosition::TopLeft | ToastPosition::TopCenter | ToastPosition::TopRight
                ) {
                    toasts.reverse();
                }

                let theme = fret_ui::Theme::global(&*cx.app).clone();
                let margin =
                    margin_override.unwrap_or_else(|| theme.metric_required("metric.padding.md"));
                let gap =
                    gap_override.unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
                let toast_padding = theme.metric_required("metric.padding.sm");
                let radius = theme.metric_required("metric.radius.md");
                let store_for_toasts = store_for_render.clone();

                let mut wrapper_layout = fret_ui::element::LayoutStyle {
                    position: fret_ui::element::PositionStyle::Absolute,
                    ..Default::default()
                };
                match position {
                    ToastPosition::TopLeft => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.left = Some(margin);
                    }
                    ToastPosition::TopCenter => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.left = Some(Px(0.0));
                        wrapper_layout.inset.right = Some(Px(0.0));
                    }
                    ToastPosition::TopRight => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                    ToastPosition::BottomLeft => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.left = Some(margin);
                    }
                    ToastPosition::BottomCenter => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.left = Some(Px(0.0));
                        wrapper_layout.inset.right = Some(Px(0.0));
                    }
                    ToastPosition::BottomRight => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                }

                let justify = match position {
                    ToastPosition::TopLeft | ToastPosition::TopCenter | ToastPosition::TopRight => {
                        fret_ui::element::MainAlign::Start
                    }
                    ToastPosition::BottomLeft
                    | ToastPosition::BottomCenter
                    | ToastPosition::BottomRight => fret_ui::element::MainAlign::End,
                };

                let align = match position {
                    ToastPosition::TopLeft | ToastPosition::BottomLeft => {
                        fret_ui::element::CrossAlign::Start
                    }
                    ToastPosition::TopCenter | ToastPosition::BottomCenter => {
                        fret_ui::element::CrossAlign::Center
                    }
                    ToastPosition::TopRight | ToastPosition::BottomRight => {
                        fret_ui::element::CrossAlign::End
                    }
                };

                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: wrapper_layout,
                        direction: fret_core::Axis::Vertical,
                        gap,
                        padding: fret_core::Edges::all(fret_core::Px(0.0)),
                        justify,
                        align,
                        wrap: false,
                    },
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(toasts.len());
                        let base_theme = theme.clone();
                        for toast in toasts {
                            let theme = base_theme.clone();
                            let store = store_for_toasts.clone();
                            let toast_id = toast.id;
                            let open = toast.open;
                            let position = position;
                            let drag_offset = toast.drag_offset;
                            let settle_from = toast.settle_from;
                            let drag_active = toast.dragging;

                            let bg_default = theme
                                .color_by_key("popover")
                                .unwrap_or_else(|| theme.color_required("popover"));
                            let fg_default = theme
                                .color_by_key("popover-foreground")
                                .unwrap_or_else(|| theme.color_required("popover-foreground"));
                            let (bg, fg) = match toast.variant {
                                ToastVariant::Default => (bg_default, fg_default),
                                ToastVariant::Destructive | ToastVariant::Error => (
                                    theme.color_by_key("destructive").unwrap_or(bg_default),
                                    theme
                                        .color_by_key("destructive-foreground")
                                        .unwrap_or(fg_default),
                                ),
                                ToastVariant::Success => (
                                    theme.color_by_key("success").unwrap_or(bg_default),
                                    theme
                                        .color_by_key("success-foreground")
                                        .unwrap_or(fg_default),
                                ),
                                ToastVariant::Info => (
                                    theme.color_by_key("info").unwrap_or(bg_default),
                                    theme.color_by_key("info-foreground").unwrap_or(fg_default),
                                ),
                                ToastVariant::Warning => (
                                    theme.color_by_key("warning").unwrap_or(bg_default),
                                    theme
                                        .color_by_key("warning-foreground")
                                        .unwrap_or(fg_default),
                                ),
                                ToastVariant::Loading => (bg_default, fg_default),
                            };
                            let border_color = theme
                                .color_by_key("border")
                                .unwrap_or_else(|| theme.color_required("border"));
                            let fg_muted = theme
                                .color_by_key("muted-foreground")
                                .unwrap_or_else(|| theme.color_required("muted-foreground"));

                            let button_bg = theme
                                .color_by_key("muted")
                                .unwrap_or_else(|| theme.color_required("muted"));
                            let button_radius = Px(6.0);
                            let button_pad_x = Px(8.0);
                            let button_pad_y = Px(4.0);

                            let close = toast.dismissible.then(|| {
                                let close_store = store.clone();
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: false,
                                        focus_ring: None,
                                        focus_ring_bounds: None,
                                        a11y: Default::default(),
                                    },
                                    move |cx, st| {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                let _ = dismiss_toast_action(
                                                    host,
                                                    close_store.clone(),
                                                    cx.window,
                                                    toast_id,
                                                );
                                            },
                                        ));

                                        let bg = if st.pressed {
                                            Some(alpha_mul(button_bg, 0.8))
                                        } else if st.hovered {
                                            Some(alpha_mul(button_bg, 0.6))
                                        } else {
                                            None
                                        };

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: fret_core::Edges {
                                                    top: button_pad_y,
                                                    right: button_pad_x,
                                                    bottom: button_pad_y,
                                                    left: button_pad_x,
                                                },
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    button_radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: "\u{00D7}".into(),
                                                    style: None,
                                                    color: Some(fg),
                                                    wrap: fret_core::TextWrap::None,
                                                    overflow: fret_core::TextOverflow::Clip,
                                                })]
                                            },
                                        )]
                                    },
                                )
                            });

                            let action = toast.action.clone().map(|action| {
                                let action_store = store.clone();
                                let cmd = action.command;
                                let label = action.label;
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: false,
                                        focus_ring: None,
                                        focus_ring_bounds: None,
                                        a11y: Default::default(),
                                    },
                                    move |cx, st| {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                host.dispatch_command(Some(cx.window), cmd.clone());
                                            },
                                        ));
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                let _ = dismiss_toast_action(
                                                    host,
                                                    action_store.clone(),
                                                    cx.window,
                                                    toast_id,
                                                );
                                            },
                                        ));

                                        let bg = if st.pressed {
                                            Some(alpha_mul(button_bg, 0.8))
                                        } else if st.hovered {
                                            Some(alpha_mul(button_bg, 0.6))
                                        } else {
                                            None
                                        };

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: fret_core::Edges {
                                                    top: button_pad_y,
                                                    right: button_pad_x,
                                                    bottom: button_pad_y,
                                                    left: button_pad_x,
                                                },
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    button_radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: label.clone(),
                                                    style: None,
                                                    color: Some(fg),
                                                    wrap: fret_core::TextWrap::None,
                                                    overflow: fret_core::TextOverflow::Clip,
                                                })]
                                            },
                                        )]
                                    },
                                )
                            });

                            let cancel = toast.cancel.clone().map(|cancel| {
                                let cancel_store = store.clone();
                                let cmd = cancel.command;
                                let label = cancel.label;
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: false,
                                        focus_ring: None,
                                        focus_ring_bounds: None,
                                        a11y: Default::default(),
                                    },
                                    move |cx, st| {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                host.dispatch_command(Some(cx.window), cmd.clone());
                                            },
                                        ));
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, cx, _reason| {
                                                let _ = dismiss_toast_action(
                                                    host,
                                                    cancel_store.clone(),
                                                    cx.window,
                                                    toast_id,
                                                );
                                            },
                                        ));

                                        let bg = if st.pressed {
                                            Some(alpha_mul(button_bg, 0.8))
                                        } else if st.hovered {
                                            Some(alpha_mul(button_bg, 0.6))
                                        } else {
                                            None
                                        };

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: fret_core::Edges {
                                                    top: button_pad_y,
                                                    right: button_pad_x,
                                                    bottom: button_pad_y,
                                                    left: button_pad_x,
                                                },
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    button_radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: label.clone(),
                                                    style: None,
                                                    color: Some(fg),
                                                    wrap: fret_core::TextWrap::None,
                                                    overflow: fret_core::TextOverflow::Clip,
                                                })]
                                            },
                                        )]
                                    },
                                )
                            });

                            let icon = match toast.variant {
                                ToastVariant::Loading => {
                                    let mut spinner = fret_ui::element::SpinnerProps::default();
                                    spinner.color = Some(fg);
                                    Some(cx.spinner_props(spinner))
                                }
                                v => toast_icon_glyph(v).map(|glyph| {
                                    cx.text_props(fret_ui::element::TextProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        text: glyph.into(),
                                        style: None,
                                        color: Some(fg),
                                        wrap: fret_core::TextWrap::None,
                                        overflow: fret_core::TextOverflow::Clip,
                                    })
                                }),
                            };

                            let icon = icon.clone();
                            let header_row = cx.flex(
                                fret_ui::element::FlexProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: toast_padding,
                                    padding: fret_core::Edges::all(fret_core::Px(0.0)),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
                                    wrap: false,
                                },
                                move |cx| {
                                    let mut row: Vec<AnyElement> = Vec::new();
                                    if let Some(icon) = icon.clone() {
                                        row.push(cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: {
                                                    let mut layout =
                                                        fret_ui::element::LayoutStyle::default();
                                                    layout.size.width =
                                                        fret_ui::element::Length::Px(Px(16.0));
                                                    layout.size.height =
                                                        fret_ui::element::Length::Px(Px(16.0));
                                                    layout
                                                },
                                                padding: fret_core::Edges::all(Px(0.0)),
                                                background: None,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(Px(0.0)),
                                            },
                                            move |_cx| vec![icon.clone()],
                                        ));
                                    }
                                    row.push(cx.text_props(fret_ui::element::TextProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        text: toast.title.clone(),
                                        style: None,
                                        color: Some(fg),
                                        wrap: fret_core::TextWrap::None,
                                        overflow: fret_core::TextOverflow::Clip,
                                    }));
                                    row.push(cx.spacer(fret_ui::element::SpacerProps {
                                        min: fret_core::Px(0.0),
                                        ..Default::default()
                                    }));

                                    let trailing_cancel = cancel.clone();
                                    let trailing_action = action.clone();
                                    let trailing_close = close.clone();
                                    if trailing_cancel.is_some()
                                        || trailing_action.is_some()
                                        || trailing_close.is_some()
                                    {
                                        row.push(cx.flex(
                                            fret_ui::element::FlexProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: toast_padding,
                                                padding: fret_core::Edges::all(Px(0.0)),
                                                justify: fret_ui::element::MainAlign::End,
                                                align: fret_ui::element::CrossAlign::Center,
                                                wrap: false,
                                                ..Default::default()
                                            },
                                            move |_cx| {
                                                let mut out = Vec::new();
                                                if let Some(el) = trailing_cancel.clone() {
                                                    out.push(el);
                                                }
                                                if let Some(el) = trailing_action.clone() {
                                                    out.push(el);
                                                }
                                                if let Some(el) = trailing_close.clone() {
                                                    out.push(el);
                                                }
                                                out
                                            },
                                        ));
                                    }
                                    row
                                },
                            );

                            let mut toast_children: Vec<AnyElement> = vec![header_row];
                            if let Some(desc) = toast.description.clone() {
                                toast_children.push(cx.text_props(fret_ui::element::TextProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    text: desc,
                                    style: None,
                                    color: Some(fg_muted),
                                    wrap: fret_core::TextWrap::Word,
                                    overflow: fret_core::TextOverflow::Clip,
                                }));
                            }

                            out.push(cx.keyed(toast_id, move |cx| {
                                let settle = crate::declarative::transition::drive_transition_with_durations_and_easing(
                                    cx,
                                    settle_from.is_some() && !drag_active,
                                    12,
                                    1,
                                    crate::headless::easing::smoothstep,
                                );
                                if settle_from.is_some()
                                    && !settle.animating
                                    && (settle.progress - 1.0).abs() <= f32::EPSILON
                                {
                                    let _ = cx
                                        .app
                                        .models_mut()
                                        .update(&store, |st| st.clear_settle(window, toast_id));
                                }

                                let settle_offset = settle_from
                                    .map(|from| {
                                        let t = (1.0 - settle.progress).clamp(0.0, 1.0);
                                        Point::new(Px(from.x.0 * t), Px(from.y.0 * t))
                                    })
                                    .unwrap_or_else(|| Point::new(Px(0.0), Px(0.0)));
                                let drag_offset = Point::new(
                                    Px(drag_offset.x.0 + settle_offset.x.0),
                                    Px(drag_offset.y.0 + settle_offset.y.0),
                                );

                                let presence =
                                    crate::OverlayController::fade_presence_with_durations(
                                        cx, open, 12, 12,
                                    );
                                let opacity = presence.opacity;
                                let slide_px = Px(16.0 * (1.0 - opacity));
                                let dx = match position {
                                    ToastPosition::TopLeft | ToastPosition::BottomLeft => {
                                        Px(-slide_px.0)
                                    }
                                    ToastPosition::TopRight | ToastPosition::BottomRight => {
                                        slide_px
                                    }
                                    ToastPosition::TopCenter | ToastPosition::BottomCenter => {
                                        Px(0.0)
                                    }
                                };
                                let dy = match position {
                                    ToastPosition::TopLeft
                                    | ToastPosition::TopCenter
                                    | ToastPosition::TopRight => Px(-slide_px.0),
                                    ToastPosition::BottomLeft
                                    | ToastPosition::BottomCenter
                                    | ToastPosition::BottomRight => slide_px,
                                };
                                let slide = Transform2D::translation(Point::new(
                                    Px(dx.0 + drag_offset.x.0),
                                    Px(dy.0 + drag_offset.y.0),
                                ));

                                let mut toast_layout = fret_ui::element::LayoutStyle::default();
                                toast_layout.size.min_width =
                                    Some(toast_min_width_override.unwrap_or(Px(280.0)));
                                toast_layout.size.max_width =
                                    Some(toast_max_width_override.unwrap_or(Px(420.0)));

                                let toast_el = cx.container(
                                    fret_ui::element::ContainerProps {
                                        layout: toast_layout,
                                        padding: fret_core::Edges::all(toast_padding),
                                        background: Some(bg),
                                        shadow: None,
                                        border: fret_core::Edges::all(fret_core::Px(1.0)),
                                        border_color: Some(border_color),
                                        focus_ring: None,
                                        focus_border_color: None,
                                        focus_within: false,
                                        corner_radii: fret_core::Corners::all(radius),
                                    },
                                    move |_cx| toast_children,
                                );

                                let store_for_hooks = store.clone();
                                let pause_store = store.clone();
                                let toast_hover = cx.hover_region(
                                    fret_ui::element::HoverRegionProps::default(),
                                    move |cx, hovered| {
                                        let hovered = hovered || drag_active;
                                        let changed =
                                            cx.with_state(ToastHoverPauseState::default, |st| {
                                                let changed = st.hovered != hovered;
                                                st.hovered = hovered;
                                                changed
                                            });

                                        if changed {
                                            if hovered {
                                                if let Ok(Some(token)) =
                                                    cx.app.models_mut().update(&pause_store, |st| {
                                                        st.pause_auto_close(cx.window, toast_id)
                                                    })
                                                {
                                                    cx.app.push_effect(
                                                        fret_runtime::Effect::CancelTimer { token },
                                                    );
                                                    cx.app.request_redraw(cx.window);
                                                }
                                            } else {
                                                let token = cx.app.next_timer_token();
                                                if let Ok(Some(after)) =
                                                    cx.app.models_mut().update(&pause_store, |st| {
                                                        st.resume_auto_close(
                                                            cx.window, toast_id, token,
                                                        )
                                                    })
                                                {
                                                    cx.app.push_effect(
                                                        fret_runtime::Effect::SetTimer {
                                                            window: Some(cx.window),
                                                            token,
                                                            after,
                                                            repeat: None,
                                                        },
                                                    );
                                                    cx.app.request_redraw(cx.window);
                                                }
                                            }
                                        }

                                        vec![cx.pointer_region(
                                            fret_ui::element::PointerRegionProps::default(),
                                            move |cx| {
                                                let store = store_for_hooks.clone();
                                                cx.pointer_region_on_pointer_down(Arc::new(
                                                    move |host, acx, down| {
                                                        let _ = host.models_mut().update(
                                                            &store,
                                                            |st| {
                                                                st.begin_drag(
                                                                    acx.window,
                                                                    toast_id,
                                                                    down.position,
                                                                )
                                                            },
                                                        );
                                                        false
                                                    },
                                                ));

                                                let store = store_for_hooks.clone();
                                                cx.pointer_region_on_pointer_move(Arc::new(
                                                    move |host, acx, mv| {
                                                        let update = host
                                                            .models_mut()
                                                            .update(&store, |st| {
                                                                st.drag_move(
                                                                    acx.window,
                                                                    toast_id,
                                                                    mv.position,
                                                                )
                                                            })
                                                            .ok()
                                                            .flatten();

                                                        let Some(update) = update else {
                                                            return false;
                                                        };

                                                        if update.capture_pointer {
                                                            host.capture_pointer();
                                                        }

                                                        if update.dragging {
                                                            host.request_redraw(acx.window);
                                                            return true;
                                                        }
                                                        false
                                                    },
                                                ));

                                                let store = store_for_hooks.clone();
                                                cx.pointer_region_on_pointer_up(Arc::new(
                                                    move |host, acx, _up| {
                                                        let end = host
                                                            .models_mut()
                                                            .update(&store, |st| {
                                                                st.end_drag(acx.window, toast_id)
                                                            })
                                                            .ok()
                                                            .flatten();

                                                        let Some(end) = end else {
                                                            return false;
                                                        };

                                                        if end.dragging {
                                                            host.release_pointer_capture();
                                                            if end.dismiss {
                                                                let _ = dismiss_toast_action(
                                                                    host,
                                                                    store.clone(),
                                                                    acx.window,
                                                                    toast_id,
                                                                );
                                                            }
                                                            host.request_redraw(acx.window);
                                                            return true;
                                                        }
                                                        false
                                                    },
                                                ));

                                                vec![toast_el.clone()]
                                            },
                                        )]
                                    },
                                );

                                cx.opacity_props(
                                    fret_ui::element::OpacityProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        opacity,
                                    },
                                    move |cx| {
                                        vec![cx.render_transform_props(
                                            fret_ui::element::RenderTransformProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                transform: slide,
                                            },
                                            move |_cx| vec![toast_hover],
                                        )]
                                    },
                                )
                            }));
                        }
                        out
                    },
                )]
            },
        );

        let has_toasts = app
            .models()
            .read(&store, |st| !st.toasts_for_window(window).is_empty())
            .unwrap_or(false);

        let key = (window, req.id);
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .toast_layers
                .entry(key)
                .or_insert_with(|| ActiveToastLayer {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                });
            entry.root_name = req.root_name.clone();
            OverlayLayer::toast(has_toasts, has_toasts).apply(ui, entry.layer);
        });
    }

    let to_hide_toast_layers: Vec<UiLayerId> =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .toast_layers
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_toast_layers.contains(id) {
                        return None;
                    }
                    Some(active.layer)
                })
                .collect()
        });

    for layer in to_hide_toast_layers {
        OverlayLayer::hide_toast().apply(ui, layer);
    }
}
