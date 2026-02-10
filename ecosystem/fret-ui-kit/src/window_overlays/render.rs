use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{
    AppWindowId, Color, FontId, FontWeight, NodeId, Point, Px, Rect, SemanticsRole, TextStyle,
    Transform2D, WindowMetricsService,
};
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

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        self.app.next_clipboard_token()
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

struct OverlayFocusHost<'a, H: UiHost> {
    ui: &'a mut UiTree<H>,
    app: &'a mut H,
    window: AppWindowId,
}

impl<'a, H: UiHost> UiActionHost for OverlayFocusHost<'a, H> {
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

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        self.app.next_clipboard_token()
    }
}

impl<'a, H: UiHost> UiFocusActionHost for OverlayFocusHost<'a, H> {
    fn request_focus(&mut self, target: fret_ui::elements::GlobalElementId) {
        if let Some(node) = fret_ui::elements::node_for_element(self.app, self.window, target) {
            self.ui.set_focus(Some(node));
        }
    }
}

pub fn render<H: UiHost + 'static>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    super::toast::drain_toast_async_queue(app);

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
                req.on_pointer_move = overlays
                    .cached_hover_overlay_pointer_move_handlers
                    .get(&(*w, *id))
                    .cloned()
                    .flatten();
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
            ui.set_layer_blocks_underlay_focus(entry.layer, open_now);
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

        let focus_layer = ui.focus().and_then(|n| ui.node_layer(n));
        let focus_in_modal_layer = layer.is_some_and(|layer| focus_layer == Some(layer));
        let focus_in_popover_layer = focus_layer.is_some_and(|focus_layer| {
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.iter().any(|((w, _id), entry)| {
                    *w == window && entry.open && entry.layer == focus_layer
                })
            })
        });
        let focus_in_layer = focus_in_modal_layer || focus_in_popover_layer;
        let enforce_focus_containment = open_now && !focus_in_layer;

        let pending_initial_focus =
            app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays
                    .modals
                    .get(&key)
                    .is_some_and(|entry| entry.pending_initial_focus)
            });

        let apply_initial_focus = (opening && !open_auto_focus_prevented) || pending_initial_focus;
        if apply_initial_focus || enforce_focus_containment {
            let focus = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays.modals.get(&key).and_then(|p| p.initial_focus)
            });
            let applied = if apply_initial_focus {
                focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus)
            } else {
                false
            };
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
        let open = req.open;
        let on_open_auto_focus = req.on_open_auto_focus.clone();
        let open_for_dismiss = open.clone();
        let on_pointer_move = req.on_pointer_move.clone();
        let on_dismiss_request = req.on_dismiss_request.clone();
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
            let focus_on_trigger = fret_ui::elements::node_for_element(app, window, trigger)
                .is_some_and(|node| focus_now == Some(node));
            let should_run_close_auto_focus = focus_in_layer
                || focus_on_trigger
                || (!focus_cleared_by_modal_scope && focus_now.is_none());

            let mut close_auto_focus_prevented = false;
            if should_run_close_auto_focus && !focus_cleared_by_modal_scope {
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
                    entry.close_auto_focus_handled = should_run_close_auto_focus;
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
            let mut focus_req = AutoFocusRequestCx::new();
            if open_now && (should_focus_initial || pending_initial_focus) {
                if let Some(on_open_auto_focus) = &on_open_auto_focus {
                    let mut host = OverlayFocusHost { ui, app, window };
                    on_open_auto_focus(
                        &mut host,
                        ActionCx {
                            window,
                            target: popover_id,
                        },
                        &mut focus_req,
                    );
                }
            }

            if focus_req.default_prevented() {
                app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                    if let Some(entry) = overlays.popovers.get_mut(&key) {
                        entry.pending_initial_focus = false;
                    }
                });
                continue;
            }

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

        // Hover overlays (e.g. HoverCard) can remain mounted during close transitions. During
        // those transitions (`open=false`, `present=true`), they must become pointer-transparent
        // to avoid blocking underlay interactions while they animate out.
        let open_now = app.models().get_copied(&req.open).unwrap_or(false);
        let interactive = req.interactive && open_now;
        let on_pointer_move = req.on_pointer_move.clone();
        let root = fret_ui::declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            move |cx| {
                if let Some(on_pointer_move) = on_pointer_move {
                    cx.dismissible_on_pointer_move(on_pointer_move);
                }
                req.children
            },
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
            ui.set_layer_wants_pointer_move_events(
                entry.layer,
                present && interactive && req.on_pointer_move.is_some(),
            );
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
        let toast_style = req.style.clone();
        let show_close_button = toast_style.show_close_button;
        let margin_override = req.margin;
        let gap_override = req.gap;
        let toast_min_width_override = req.toast_min_width;
        let toast_max_width_override = req.toast_max_width;
        let toaster_id = req.toaster_id.clone();
        let visible_toasts = req.visible_toasts.max(1);
        let expand_by_default = req.expand_by_default;
        let rich_colors_default = req.rich_colors;
        let invert_default = req.invert;
        let toaster_key = req.id;
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

                let mut toasts = toasts;
                if let Some(toaster_id) = toaster_id.as_ref() {
                    toasts.retain(|t| t.toaster_id.as_ref() == Some(toaster_id));
                } else {
                    toasts.retain(|t| t.toaster_id.is_none());
                }

                if toasts.is_empty() {
                    return Vec::new();
                }

                cx.key_on_key_down_for(
                    cx.root_id(),
                    Arc::new(move |host, acx, down| {
                        if down.key == fret_core::KeyCode::Escape {
                            host.dispatch_command(
                                Some(acx.window),
                                fret_runtime::CommandId::from(
                                    super::TOAST_VIEWPORT_RESTORE_COMMAND,
                                ),
                            );
                            return true;
                        }
                        false
                    }),
                );

                let (margin, gap, radius) = {
                    let theme = fret_ui::Theme::global(&*cx.app);
                    let margin = margin_override.unwrap_or_else(|| Px(24.0));
                    let gap = gap_override.unwrap_or_else(|| Px(14.0));
                    let radius = toast_style
                        .container_radius
                        .unwrap_or_else(|| theme.metric_required("metric.radius.md"));
                    (margin, gap, radius)
                };

                let toast_padding = toast_style
                    .container_padding
                    .unwrap_or(fret_core::Edges::all(Px(16.0)));
                let toast_width = toast_max_width_override
                    .or(toast_min_width_override)
                    .unwrap_or(Px(356.0));

                let ring = fret_ui::element::RingStyle {
                    placement: fret_ui::element::RingPlacement::Outset,
                    width: Px(2.0),
                    offset: Px(0.0),
                    color: Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.2,
                    },
                    offset_color: None,
                    corner_radii: fret_core::Corners::all(radius),
                };

                let shadow = toast_style.shadow.or_else(|| {
                    Some(fret_ui::element::ShadowStyle {
                        primary: fret_ui::element::ShadowLayerStyle {
                            color: Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.1,
                            },
                            offset_x: Px(0.0),
                            offset_y: Px(4.0),
                            blur: Px(12.0),
                            spread: Px(0.0),
                        },
                        secondary: None,
                        corner_radii: fret_core::Corners::all(radius),
                    })
                });

                let toaster_state = cx
                    .app
                    .models()
                    .read(&store_for_render, |st| {
                        st.toaster_state(window, toaster_key)
                    })
                    .unwrap_or_default();
                let hotkey_expanded = toaster_state.hotkey_expanded;
                let interacting = toaster_state.interacting;

                let total_toasts = toasts.len();
                let mut positions: Vec<ToastPosition> = vec![position];
                for toast in &toasts {
                    if let Some(p) = toast.position {
                        if !positions.contains(&p) {
                            positions.push(p);
                        }
                    }
                }

                let store_for_hover = store_for_render.clone();
                let store_for_render_state = store_for_render.clone();
                let toasts_for_render = toasts.clone();
                vec![cx.hover_region(
                    fret_ui::element::HoverRegionProps::default(),
                    move |cx, hovered| {
                        let hovered = hovered || interacting;
                        let _ = cx
                            .app
                            .models_mut()
                            .update(&store_for_hover, |st| {
                                st.set_toaster_hovered(cx.window, toaster_key, hovered)
                            });

                        let expanded = expand_by_default
                            || (total_toasts > 1 && (hotkey_expanded || hovered));
                        if total_toasts <= 1 && hotkey_expanded {
                            let _ = cx.app.models_mut().update(&store_for_hover, |st| {
                                st.set_toaster_hotkey_expanded(cx.window, toaster_key, false)
                            });
                        }

                        let focus_trap = cx
                            .app
                            .models()
                            .read(&store_for_render_state, |st| {
                                st.toaster_state(cx.window, toaster_key).hotkey_expanded
                            })
                            .unwrap_or(false);

                        vec![cx.focus_scope(
                            fret_ui::element::FocusScopeProps {
                                layout: fret_ui::element::LayoutStyle::default(),
                                trap_focus: focus_trap,
                            },
                            move |cx| {
                                let mut stacks: Vec<AnyElement> = Vec::new();
                                for stack_position in positions.clone() {
                                    let is_base = stack_position == position;
                                    let mut stack_toasts: Vec<ToastEntry> = toasts_for_render
                                        .iter()
                                        .filter(|t| {
                                            if is_base {
                                                t.position.is_none()
                                                    || t.position == Some(position)
                                            } else {
                                                t.position == Some(stack_position)
                                            }
                                        })
                                        .cloned()
                                        .collect();
                                    if stack_toasts.is_empty() {
                                        continue;
                                    }
                                    // Newest-first to match Sonner's front-toast semantics.
                                    stack_toasts.reverse();

                                    let is_top = matches!(
                                        stack_position,
                                        ToastPosition::TopLeft
                                            | ToastPosition::TopCenter
                                            | ToastPosition::TopRight
                                    );
                                    let lift = if is_top { 1.0 } else { -1.0 };

                                    let estimate_height = |toast: &ToastEntry| -> Px {
                                        if toast.description.is_some() {
                                            toast_style
                                                .two_line_min_height
                                                .unwrap_or(Px(72.0))
                                        } else {
                                            toast_style
                                                .single_line_min_height
                                                .unwrap_or(Px(52.0))
                                        }
                                    };

                                    let mut offsets: Vec<Px> =
                                        Vec::with_capacity(stack_toasts.len());
                                    if expanded {
                                        let mut acc = Px(0.0);
                                        for toast in &stack_toasts {
                                            offsets.push(acc);
                                            let h = toast
                                                .measured_height
                                                .unwrap_or_else(|| estimate_height(toast));
                                            acc = Px(acc.0 + h.0 + gap.0);
                                        }
                                    } else {
                                        for i in 0..stack_toasts.len() {
                                            offsets.push(Px(gap.0 * i as f32));
                                        }
                                    }

                                    let front_height = stack_toasts[0]
                                        .measured_height
                                        .unwrap_or_else(|| estimate_height(&stack_toasts[0]));

                                    let window_w = bounds.size.width;
                                    let x = match stack_position {
                                        ToastPosition::TopLeft | ToastPosition::BottomLeft => margin,
                                        ToastPosition::TopRight | ToastPosition::BottomRight => {
                                            Px((window_w.0 - margin.0 - toast_width.0).max(0.0))
                                        }
                                        ToastPosition::TopCenter
                                        | ToastPosition::BottomCenter => {
                                            Px(((window_w.0 - toast_width.0) * 0.5).max(0.0))
                                        }
                                    };

                                    let mut wrapper_layout = fret_ui::element::LayoutStyle {
                                        position: fret_ui::element::PositionStyle::Absolute,
                                        ..Default::default()
                                    };
                                    wrapper_layout.inset.left = Some(x);
                                    wrapper_layout.size.width =
                                        fret_ui::element::Length::Px(toast_width);
                                    if is_top {
                                        wrapper_layout.inset.top = Some(margin);
                                    } else {
                                        wrapper_layout.inset.bottom = Some(margin);
                                    }

                                    let store_for_toasts = store_for_render_state.clone();
                                    let shadow = shadow;
                                    stacks.push(cx.stack_props(
                                        fret_ui::element::StackProps {
                                            layout: wrapper_layout,
                                        },
                                        move |cx| {
                                            let mut out: Vec<AnyElement> =
                                                Vec::with_capacity(stack_toasts.len());

                                            for idx in (0..stack_toasts.len()).rev() {
                                                let toast = stack_toasts[idx].clone();
                                                let toast_id = toast.id;
                                                let toast_visible =
                                                    expanded || idx < visible_toasts;
                                                let toast_content_visible =
                                                    expanded || idx == 0 || expand_by_default;
                                                let toast_height_override =
                                                    (!toast_content_visible)
                                                        .then_some(front_height);
                                                let stack_offset_y =
                                                    Px(lift * offsets[idx].0);
                                                let stack_scale = if expanded || idx == 0 {
                                                    1.0
                                                } else {
                                                    1.0 + 0.05 * (idx as f32)
                                                };

                                                let store = store_for_toasts.clone();
                                                let open = toast.open;
                                                let drag_offset = toast.drag_offset;
                                                let settle_from = toast.settle_from;
                                                let drag_active = toast.dragging;

                                                let rich_colors = toast
                                                    .rich_colors
                                                    .unwrap_or(rich_colors_default);
                                                let invert = toast.invert || invert_default;
                                                let test_id = toast.test_id.clone();

                                                let (bg, fg, border_color, fg_muted, button_bg): (
                                                    Color,
                                                    Color,
                                                    Color,
                                                    Color,
                                                    Color,
                                                ) = {
                                                        let theme =
                                                            fret_ui::Theme::global(&*cx.app);

                                                        let bg_default = theme
                                                            .color_by_key("popover")
                                                            .unwrap_or_else(|| {
                                                                theme.color_required("popover")
                                                            });
                                                        let fg_default = theme
                                                            .color_by_key("popover-foreground")
                                                            .unwrap_or_else(|| {
                                                                theme.color_required(
                                                                    "popover-foreground",
                                                                )
                                                            });

                                                        let (mut bg, mut fg) = if rich_colors {
                                                            match toast.variant {
                                                                ToastVariant::Default => {
                                                                    (bg_default, fg_default)
                                                                }
                                                                ToastVariant::Destructive
                                                                | ToastVariant::Error => (
                                                                    theme
                                                                        .color_by_key("destructive")
                                                                        .unwrap_or(bg_default),
                                                                    theme
                                                                        .color_by_key(
                                                                            "destructive-foreground",
                                                                        )
                                                                        .unwrap_or(fg_default),
                                                                ),
                                                                ToastVariant::Success => (
                                                                    theme
                                                                        .color_by_key("success")
                                                                        .unwrap_or(bg_default),
                                                                    theme
                                                                        .color_by_key(
                                                                            "success-foreground",
                                                                        )
                                                                        .unwrap_or(fg_default),
                                                                ),
                                                                ToastVariant::Info => (
                                                                    theme
                                                                        .color_by_key("info")
                                                                        .unwrap_or(bg_default),
                                                                    theme
                                                                        .color_by_key(
                                                                            "info-foreground",
                                                                        )
                                                                        .unwrap_or(fg_default),
                                                                ),
                                                                ToastVariant::Warning => (
                                                                    theme
                                                                        .color_by_key("warning")
                                                                        .unwrap_or(bg_default),
                                                                    theme
                                                                        .color_by_key(
                                                                            "warning-foreground",
                                                                        )
                                                                        .unwrap_or(fg_default),
                                                                ),
                                                                ToastVariant::Loading => {
                                                                    (bg_default, fg_default)
                                                                }
                                                            }
                                                        } else {
                                                            (bg_default, fg_default)
                                                        };

                                                        if invert {
                                                            std::mem::swap(&mut bg, &mut fg);
                                                        }

                                                        let border_color = theme
                                                            .color_by_key("border")
                                                            .unwrap_or_else(|| {
                                                                theme.color_required("border")
                                                            });
                                                        let fg_muted = theme
                                                            .color_by_key("muted-foreground")
                                                            .unwrap_or_else(|| {
                                                                theme.color_required(
                                                                    "muted-foreground",
                                                                )
                                                            });
                                                        let button_bg = theme
                                                            .color_by_key("muted")
                                                            .unwrap_or_else(|| {
                                                                theme.color_required("muted")
                                                            });

                                                        (bg, fg, border_color, fg_muted, button_bg)
                                                    };

                                                let button_radius = Px(6.0);
                                                let button_pad_x = Px(8.0);
                                                let button_pad_y = Px(4.0);

                                                let close = (toast.dismissible
                                                    && show_close_button
                                                    && toast_content_visible)
                                                .then(|| {
                                                    let close_store = store.clone();
                                                    cx.pressable(
                                                        fret_ui::element::PressableProps {
                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                            enabled: true,
                                                            focusable: true,
                                                            focus_ring: Some(ring),
                                                            focus_ring_bounds: None,
                                                            key_activation: Default::default(),
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
                                                                    corner_radii: fret_core::Corners::all(button_radius),
                                                                    snap_to_device_pixels: false,
                                                                },
                                                                move |cx| {
                                                                    vec![cx.text_props(fret_ui::element::TextProps {
                                                                        layout: fret_ui::element::LayoutStyle::default(),
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

                                                let action = (toast_content_visible)
                                                    .then(|| toast.action.clone())
                                                    .flatten()
                                                    .map(|action| {
                                                        let action_store = store.clone();
                                                        let cmd = action.command;
                                                        let label = action.label;
                                                        cx.pressable(
                                                            fret_ui::element::PressableProps {
                                                                layout: fret_ui::element::LayoutStyle::default(),
                                                                enabled: true,
                                                                focusable: true,
                                                                focus_ring: Some(ring),
                                                                focus_ring_bounds: None,
                                                                key_activation: Default::default(),
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

                                                                let button_bg = if st.pressed {
                                                                    Some(alpha_mul(fg, 0.8))
                                                                } else if st.hovered {
                                                                    Some(alpha_mul(fg, 0.9))
                                                                } else {
                                                                    Some(fg)
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
                                                                        background: button_bg,
                                                                        shadow: None,
                                                                        border: fret_core::Edges::all(Px(0.0)),
                                                                        border_color: None,
                                                                        focus_ring: None,
                                                                        focus_border_color: None,
                                                                        focus_within: false,
                                                                        corner_radii: fret_core::Corners::all(button_radius),
                                                                        snap_to_device_pixels: false,
                                                                    },
                                                                    move |cx| {
                                                                        vec![cx.text_props(fret_ui::element::TextProps {
                                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                                            text: label.clone(),
                                                                            style: None,
                                                                            color: Some(bg),
                                                                            wrap: fret_core::TextWrap::None,
                                                                            overflow: fret_core::TextOverflow::Clip,
                                                                        })]
                                                                    },
                                                                )]
                                                            },
                                                        )
                                                    });

                                                let cancel = (toast_content_visible)
                                                    .then(|| toast.cancel.clone())
                                                    .flatten()
                                                    .map(|cancel| {
                                                        let cancel_store = store.clone();
                                                        let cmd = cancel.command;
                                                        let label = cancel.label;
                                                        cx.pressable(
                                                            fret_ui::element::PressableProps {
                                                                layout: fret_ui::element::LayoutStyle::default(),
                                                                enabled: true,
                                                                focusable: true,
                                                                focus_ring: Some(ring),
                                                                focus_ring_bounds: None,
                                                                key_activation: Default::default(),
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
                                                                    Some(alpha_mul(button_bg, 0.3))
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
                                                                        corner_radii: fret_core::Corners::all(button_radius),
                                                                        snap_to_device_pixels: false,
                                                                    },
                                                                    move |cx| {
                                                                        vec![cx.text_props(fret_ui::element::TextProps {
                                                                            layout: fret_ui::element::LayoutStyle::default(),
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

                                                let toast_inner = if toast_content_visible {
                                                    // Sonner toast typography (`sonner@2.x`):
                                                    // - toast box: `font-size: 13px`
                                                    // - title: `font-weight: 500`, `line-height: 1.5`
                                                    // - description: `font-weight: 400`, `line-height: 1.4`
                                                    let title_style = TextStyle {
                                                        font: FontId::default(),
                                                        size: Px(13.0),
                                                        weight: FontWeight(500),
                                                        slant: Default::default(),
                                                        line_height: Some(Px(13.0 * 1.5)),
                                                        letter_spacing_em: None,
                                                    };
                                                    let desc_style = TextStyle {
                                                        font: FontId::default(),
                                                        size: Px(13.0),
                                                        weight: FontWeight(400),
                                                        slant: Default::default(),
                                                        line_height: Some(Px(13.0 * 1.4)),
                                                        letter_spacing_em: None,
                                                    };

                                                    let title = cx.text_props(fret_ui::element::TextProps {
                                                        layout: fret_ui::element::LayoutStyle::default(),
                                                        text: toast.title.clone(),
                                                        style: Some(title_style),
                                                        color: Some(fg),
                                                        wrap: fret_core::TextWrap::None,
                                                        overflow: fret_core::TextOverflow::Clip,
                                                    });

                                                    let mut content_children: Vec<AnyElement> = vec![title];
                                                    if let Some(desc) = toast.description.clone() {
                                                        content_children.push(cx.text_props(fret_ui::element::TextProps {
                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                            text: desc,
                                                            style: Some(desc_style),
                                                            color: Some(fg_muted),
                                                            wrap: fret_core::TextWrap::Word,
                                                            overflow: fret_core::TextOverflow::Clip,
                                                        }));
                                                    }

                                                    let content = cx.column(
                                                        fret_ui::element::ColumnProps {
                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                            gap: Px(2.0),
                                                            padding: fret_core::Edges::all(Px(0.0)),
                                                            justify: fret_ui::element::MainAlign::Start,
                                                            align: fret_ui::element::CrossAlign::Start,
                                                        },
                                                        move |_cx| content_children,
                                                    );

                                                    let icon = icon.clone();
                                                    let cancel = cancel.clone();
                                                    let action = action.clone();
                                                    let close = close.clone();
                                                    Some(cx.flex(
                                                        fret_ui::element::FlexProps {
                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                            direction: fret_core::Axis::Horizontal,
                                                            gap: Px(6.0),
                                                            padding: fret_core::Edges::all(Px(0.0)),
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
                                                                            let mut layout = fret_ui::element::LayoutStyle::default();
                                                                            layout.size.width = fret_ui::element::Length::Px(Px(16.0));
                                                                            layout.size.height = fret_ui::element::Length::Px(Px(16.0));
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
                                                                        snap_to_device_pixels: false,
                                                                    },
                                                                    move |_cx| vec![icon.clone()],
                                                                ));
                                                            }

                                                            row.push(content.clone());
                                                            row.push(cx.spacer(fret_ui::element::SpacerProps { min: Px(0.0), ..Default::default() }));
                                                            if let Some(el) = cancel.clone() { row.push(el); }
                                                            if let Some(el) = action.clone() { row.push(el); }
                                                            if let Some(el) = close.clone() { row.push(el); }
                                                            row
                                                        },
                                                    ))
                                                } else {
                                                    None
                                                };

                                                let toast_children: Vec<AnyElement> = toast_inner.into_iter().collect();

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

                                                    let presence = crate::OverlayController::fade_presence_with_durations(
                                                        cx,
                                                        open,
                                                        toast_style.open_ticks,
                                                        toast_style.close_ticks,
                                                    );
                                                    let mut opacity = presence.opacity;
                                                    if !toast_visible {
                                                        opacity = 0.0;
                                                    }

                                                    let slide_px = Px(toast_style.slide_distance.0 * (1.0 - presence.opacity));
                                                    let dx = match stack_position {
                                                        ToastPosition::TopLeft | ToastPosition::BottomLeft => Px(-slide_px.0),
                                                        ToastPosition::TopRight | ToastPosition::BottomRight => slide_px,
                                                        ToastPosition::TopCenter | ToastPosition::BottomCenter => Px(0.0),
                                                    };
                                                    let dy = if is_top { Px(-slide_px.0) } else { slide_px };
                                                    let transform = Transform2D::translation(Point::new(
                                                        Px(dx.0 + drag_offset.x.0),
                                                        Px(dy.0 + drag_offset.y.0 + stack_offset_y.0),
                                                    ))
                                                    .compose(Transform2D::scale_uniform(stack_scale));

                                                    let mut toast_layout = fret_ui::element::LayoutStyle {
                                                        position: fret_ui::element::PositionStyle::Absolute,
                                                        ..Default::default()
                                                    };
                                                    toast_layout.inset.left = Some(Px(0.0));
                                                    if is_top {
                                                        toast_layout.inset.top = Some(Px(0.0));
                                                    } else {
                                                        toast_layout.inset.bottom = Some(Px(0.0));
                                                    }
                                                    toast_layout.size.width = fret_ui::element::Length::Fill;
                                                    toast_layout.size.min_width = Some(toast_min_width_override.unwrap_or(Px(280.0)));
                                                    toast_layout.size.max_width = Some(toast_max_width_override.unwrap_or(Px(420.0)));
                                                    if let Some(h) = toast_height_override {
                                                        toast_layout.size.height = fret_ui::element::Length::Px(h);
                                                    }

                                                    let a11y_test_id: Arc<str> = test_id.clone().unwrap_or_else(|| {
                                                        Arc::<str>::from(format!("toast-entry-{}", toast_id.0))
                                                    });

                                                    let mut pressable_layout = fret_ui::element::LayoutStyle::default();
                                                    pressable_layout.size.width = fret_ui::element::Length::Fill;
                                                    if toast_height_override.is_some() {
                                                        pressable_layout.size.height = fret_ui::element::Length::Fill;
                                                    }

                                                    let store_for_measure = store.clone();
                                                    let toast_pressable = cx.pressable_with_id(
                                                        fret_ui::element::PressableProps {
                                                            layout: pressable_layout,
                                                            enabled: toast_visible,
                                                            focusable: toast_visible,
                                                            focus_ring: Some(ring),
                                                            focus_ring_bounds: None,
                                                            key_activation: Default::default(),
                                                            a11y: fret_ui::element::PressableA11y {
                                                                role: Some(SemanticsRole::Alert),
                                                                test_id: Some(a11y_test_id),
                                                                ..Default::default()
                                                            },
                                                        },
                                                        move |cx, _st, id| {
                                                            if let Some(b) = cx.last_bounds_for_element(id) {
                                                                let h = b.size.height;
                                                                if let Ok(changed) = cx.app.models_mut().update(&store_for_measure, |st| {
                                                                    st.set_toast_measured_height(cx.window, toast_id, h)
                                                                }) {
                                                                    if changed {
                                                                        cx.app.request_redraw(cx.window);
                                                                    }
                                                                }
                                                            }

                                                            let toast_el = cx.container(
                                                                fret_ui::element::ContainerProps {
                                                                    layout: fret_ui::element::LayoutStyle::default(),
                                                                    padding: toast_padding,
                                                                    background: Some(bg),
                                                                    shadow,
                                                                    border: fret_core::Edges::all(fret_core::Px(1.0)),
                                                                    border_color: Some(border_color),
                                                                    focus_ring: None,
                                                                    focus_border_color: None,
                                                                    focus_within: false,
                                                                    corner_radii: fret_core::Corners::all(radius),
                                                                    snap_to_device_pixels: false,
                                                                },
                                                                move |cx| {
                                                                    vec![cx.flex(
                                                                        fret_ui::element::FlexProps {
                                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                                            direction: fret_core::Axis::Vertical,
                                                                            gap: fret_core::Px(4.0),
                                                                            padding: fret_core::Edges::all(fret_core::Px(0.0)),
                                                                            justify: fret_ui::element::MainAlign::Start,
                                                                            align: fret_ui::element::CrossAlign::Stretch,
                                                                            wrap: false,
                                                                        },
                                                                        move |_cx| toast_children.clone(),
                                                                    )]
                                                                },
                                                            );

                                                            vec![toast_el]
                                                        },
                                                    );

                                                    let pause_store = store.clone();
                                                    let store_for_hooks = store.clone();
                                                    let toaster_store = store.clone();
                                                    let toast_pressable_for_hover = toast_pressable.clone();
                                                    let toast_hover = cx.hover_region(
                                                        fret_ui::element::HoverRegionProps::default(),
                                                        move |cx, hovered| {
                                                            let hovered = hovered || drag_active;
                                                            let changed = cx.with_state(ToastHoverPauseState::default, |st| {
                                                                let changed = st.hovered != hovered;
                                                                st.hovered = hovered;
                                                                changed
                                                            });

                                                            if changed {
                                                                if hovered {
                                                                    if let Ok(Some(token)) = cx.app.models_mut().update(&pause_store, |st| {
                                                                        st.pause_auto_close(cx.window, toast_id)
                                                                    }) {
                                                                        cx.app.push_effect(fret_runtime::Effect::CancelTimer { token });
                                                                        cx.app.request_redraw(cx.window);
                                                                    }
                                                                } else {
                                                                    let token = cx.app.next_timer_token();
                                                                    if let Ok(Some(after)) = cx.app.models_mut().update(&pause_store, |st| {
                                                                        st.resume_auto_close(cx.window, toast_id, token)
                                                                    }) {
                                                                        cx.app.push_effect(fret_runtime::Effect::SetTimer {
                                                                            window: Some(cx.window),
                                                                            token,
                                                                            after,
                                                                            repeat: None,
                                                                        });
                                                                        cx.app.request_redraw(cx.window);
                                                                    }
                                                                }
                                                            }

                                                            let store_for_down = store_for_hooks.clone();
                                                            let toaster_store_for_down = toaster_store.clone();
                                                            let store_for_move = store_for_hooks.clone();
                                                            let store_for_up = store_for_hooks.clone();
                                                            let toaster_store_for_up = toaster_store.clone();
                                                            let toaster_store_for_cancel = toaster_store.clone();
                                                            let toast_pressable_for_pointer = toast_pressable_for_hover.clone();

                                                            vec![cx.pointer_region(
                                                                fret_ui::element::PointerRegionProps::default(),
                                                                move |cx| {
                                                                    cx.pointer_region_on_pointer_down(Arc::new(
                                                                        move |host, acx, down| {
                                                                            let _ = host.models_mut().update(&toaster_store_for_down, |st| {
                                                                                st.set_toaster_interacting(acx.window, toaster_key, true)
                                                                            });
                                                                            let _ = host.models_mut().update(&store_for_down, |st| {
                                                                                st.begin_drag(acx.window, toast_id, down.position)
                                                                            });
                                                                            false
                                                                        },
                                                                    ));

                                                                    cx.pointer_region_on_pointer_move(Arc::new(
                                                                        move |host, acx, mv| {
                                                                            let update = host
                                                                                .models_mut()
                                                                                .update(&store_for_move, |st| st.drag_move(acx.window, toast_id, mv.position))
                                                                                .ok()
                                                                                .flatten();

                                                                            let Some(update) = update else { return false; };
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

                                                                    cx.pointer_region_on_pointer_up(Arc::new(
                                                                        move |host, acx, _up| {
                                                                            let _ = host.models_mut().update(&toaster_store_for_up, |st| {
                                                                                st.set_toaster_interacting(acx.window, toaster_key, false)
                                                                            });

                                                                            let end = host
                                                                                .models_mut()
                                                                                .update(&store_for_up, |st| st.end_drag(acx.window, toast_id))
                                                                                .ok()
                                                                                .flatten();

                                                                            let Some(end) = end else { return false; };
                                                                            if end.dragging {
                                                                                host.release_pointer_capture();
                                                                                if end.dismiss {
                                                                                    let _ = dismiss_toast_action(host, store_for_up.clone(), acx.window, toast_id);
                                                                                }
                                                                                host.request_redraw(acx.window);
                                                                                return true;
                                                                            }
                                                                            false
                                                                        },
                                                                    ));

                                                                    cx.pointer_region_on_pointer_cancel(Arc::new(
                                                                        move |host, acx, _cancel| {
                                                                            let _ = host.models_mut().update(&toaster_store_for_cancel, |st| {
                                                                                st.set_toaster_interacting(acx.window, toaster_key, false)
                                                                            });
                                                                            host.release_pointer_capture();
                                                                            true
                                                                        },
                                                                    ));

                                                                    vec![toast_pressable_for_pointer.clone()]
                                                                },
                                                            )]
                                                        },
                                                    );

                                                    let content = cx.opacity_props(
                                                        fret_ui::element::OpacityProps {
                                                            layout: fret_ui::element::LayoutStyle::default(),
                                                            opacity,
                                                        },
                                                        move |cx| {
                                                            vec![cx.render_transform_props(
                                                                fret_ui::element::RenderTransformProps {
                                                                    layout: toast_layout,
                                                                    transform,
                                                                },
                                                                move |_cx| vec![toast_hover],
                                                            )]
                                                        },
                                                    );

                                                    cx.interactivity_gate(
                                                        true,
                                                        toast_visible,
                                                        move |_cx| vec![content],
                                                    )
                                                }));
                                            }
                                            out
                                        },
                                    ));
                                }
                                stacks
                            },
                        )]
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

    // Stable, policy-owned z-order correction.
    //
    // Overlay layers are installed lazily; without an explicit ordering rule, the final z-order
    // can depend on first-creation order (e.g. a long-lived toast layer created before the first
    // modal, causing toasts to render under modal barriers).
    //
    // Keep the ordering deterministic and Radix/GPUI-aligned:
    // base < popovers < modals < hover/tooltips < toasts.
    let layer_priorities: HashMap<UiLayerId, u8> =
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            let mut out: HashMap<UiLayerId, u8> = HashMap::new();

            for ((w, _id), active) in overlays.popovers.iter() {
                if *w == window {
                    out.insert(active.layer, 10);
                }
            }
            for ((w, _id), active) in overlays.modals.iter() {
                if *w == window {
                    out.insert(active.layer, 20);
                }
            }
            for ((w, _id), active) in overlays.hover_overlays.iter() {
                if *w == window {
                    out.insert(active.layer, 30);
                }
            }
            for ((w, _id), active) in overlays.tooltips.iter() {
                if *w == window {
                    out.insert(active.layer, 40);
                }
            }
            for ((w, _id), active) in overlays.toast_layers.iter() {
                if *w == window {
                    out.insert(active.layer, 50);
                }
            }
            out
        });

    let mut indexed: Vec<(usize, UiLayerId)> = ui
        .layer_ids_in_paint_order()
        .iter()
        .copied()
        .enumerate()
        .collect();
    indexed.sort_by_key(|(idx, layer)| (layer_priorities.get(layer).copied().unwrap_or(0), *idx));
    let next_order: Vec<UiLayerId> = indexed.into_iter().map(|(_, id)| id).collect();
    ui.reorder_layers_in_paint_order(next_order);
}
