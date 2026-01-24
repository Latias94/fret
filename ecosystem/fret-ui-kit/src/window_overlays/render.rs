use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{AppWindowId, Color, NodeId, Point, Px, Rect, Transform2D, WindowMetricsService};
use fret_runtime::DRAG_KIND_DOCK_PANEL;
use fret_ui::action::{ActionCx, DismissReason, UiActionHostAdapter, UiActionHostExt};
use fret_ui::declarative;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{Invalidation, UiHost, UiTree};

use crate::primitives::dismissable_layer as dismissable_layer_prim;
use crate::primitives::focus_scope as focus_scope_prim;

use super::state::{
    ActiveHoverOverlay, ActiveModal, ActivePopover, ActiveToastLayer, ActiveTooltip, OverlayLayer,
    WindowOverlays,
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

pub fn render<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
) {
    let focused_now = app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.focused(window));
    let scale_factor_now = app
        .global::<WindowMetricsService>()
        .and_then(|svc| svc.scale_factor(window));

    let (focus_lost, resized) =
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

            (focus_lost, resized)
        });

    let dock_drag_affects_window = app.drag(fret_core::PointerId(0)).is_some_and(|d| {
        d.kind == DRAG_KIND_DOCK_PANEL && (d.source_window == window || d.current_window == window)
    });

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
    let toast_request_ids: HashSet<GlobalElementId> = toast_requests.iter().map(|r| r.id).collect();

    let (extra_modals, extra_popovers, extra_hover_overlays, extra_tooltips, extra_toasts) = app
        .with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
            let mut modals: Vec<ModalRequest> = Vec::new();
            let mut popovers: Vec<DismissiblePopoverRequest> = Vec::new();
            let hover_overlays: Vec<HoverOverlayRequest> = Vec::new();
            let tooltips: Vec<TooltipRequest> = Vec::new();
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

            // Hover overlays currently have no authoritative open/present model (unlike
            // popovers/modals). Re-emitting cached hover overlays would therefore keep them alive
            // even after the source subtree stops requesting them, which breaks Radix-style
            // unmount expectations (e.g. `HoverCard` closing on blur).
            //
            // Treat hover overlays as strictly per-frame requests until we introduce an explicit
            // presence contract for `HoverOverlayRequest`.

            // Tooltips have the same challenge as hover overlays: without an explicit open/present
            // model, cached synthesis can keep tooltips alive even after their producer stops
            // requesting them (e.g. when hoverable content becomes disabled).
            //
            // Keep tooltips strictly per-frame until `TooltipRequest` has an authoritative open
            // signal that survives view caching.

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
                    Arc::new(move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                    })
                }));
                children
            },
        );

        let key = (window, modal_id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        let mut pending_initial_focus = false;
        let mut layer: Option<UiLayerId> = None;
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
            let mut created = false;
            let entry = overlays.modals.entry(key).or_insert_with(|| {
                created = true;
                ActiveModal {
                    layer: ui.push_overlay_root_ex(root, true, true),
                    root_name: root_name.clone(),
                    trigger,
                    initial_focus,
                    open: false,
                    restore_focus: None,
                    pending_initial_focus: false,
                }
            });
            entry.root_name = root_name.clone();
            entry.trigger = trigger;
            entry.initial_focus = initial_focus;
            layer = Some(entry.layer);

            // For modal overlays, `present` is the authority for whether the barrier is active.
            OverlayLayer::modal(true, open_now).apply(ui, entry.layer);

            // Radix-style focus restore for close transitions:
            // when a modal overlay closes but remains mounted (`present=true`) for an exit
            // transition, restore focus deterministically if focus is currently inside the modal
            // layer (or has been cleared by the hide pass).
            //
            // This mirrors the non-modal close-edge restore logic below, but is safe for modals as
            // well: underlay focus cannot change while the barrier is installed.
            let focus_now = ui.focus();
            let closing = entry.open && !open_now;
            if closing {
                let focus_in_layer =
                    focus_now.is_some_and(|n| ui.node_layer(n) == Some(entry.layer));
                if focus_now.is_none() || focus_in_layer {
                    if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                        ui,
                        app,
                        window,
                        entry.trigger,
                        entry.restore_focus,
                    ) {
                        ui.set_focus(Some(node));
                    }
                }
            }

            let opening = open_now && (!entry.open || created);
            if opening {
                should_focus_initial = true;
                entry.restore_focus = restore_focus;
                entry.pending_initial_focus = true;
            }
            entry.open = open_now;
            pending_initial_focus = entry.pending_initial_focus;
        });

        let focus_in_layer = layer.is_some_and(|layer| {
            ui.focus()
                .is_some_and(|n| ui.node_layer(n).is_some_and(|lid| lid == layer))
        });
        let enforce_focus_containment = open_now && !focus_in_layer;

        if should_focus_initial || pending_initial_focus || enforce_focus_containment {
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
        // Radix-aligned default: treat the trigger as an implicit DismissableLayerBranch so clicks
        // on the trigger don't count as "outside press" for this overlay layer.
        //
        // Without this, a trigger click while the popover is open can:
        // - first close the popover via the outside-press observer pass, then
        // - re-open it when the trigger toggles the open model on activate.
        let dismissable_branch_nodes =
            dismissable_layer_prim::resolve_branch_nodes_for_trigger_and_elements(
                app,
                window,
                req.trigger,
                &req.dismissable_branches,
            );

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
        let disable_outside_pointer_events = req.disable_outside_pointer_events;
        let open = req.open;
        let open_for_dismiss = open.clone();
        let on_pointer_move = req.on_pointer_move.clone();
        let on_dismiss_request = req.on_dismiss_request.clone();
        let on_dismiss_request_for_root = on_dismiss_request.clone();
        let children = req.children;

        let pointer_barrier_root = disable_outside_pointer_events.then(|| {
            let root_name = format!("{}::pointer_barrier", root_name);
            fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                &root_name,
                |_cx| Vec::new(),
            )
        });

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
                    Arc::new(move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(&open_for_dismiss, |v| *v = false);
                    })
                }));
                children
            },
        );

        let key = (window, popover_id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        app.with_global_mut_untracked(WindowOverlays::default, |overlays, app| {
            let mut created = false;
            let entry = overlays.popovers.entry(key).or_insert_with(|| {
                created = true;
                let pointer_barrier_layer = pointer_barrier_root.map(|root| {
                    // A hit-test-inert, non-visual layer that blocks underlay pointer events while
                    // the popover is open (Radix `disableOutsidePointerEvents` outcome).
                    ui.push_overlay_root_ex(root, true, false)
                });
                ActivePopover {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    pointer_barrier_layer,
                    root_name: root_name.clone(),
                    trigger,
                    initial_focus,
                    consume_outside_pointer_events,
                    disable_outside_pointer_events,
                    open: false,
                    restore_focus: None,
                    last_focus: focus_now,
                }
            });
            entry.root_name = root_name.clone();
            entry.trigger = trigger;
            entry.initial_focus = initial_focus;
            entry.consume_outside_pointer_events = consume_outside_pointer_events;
            entry.disable_outside_pointer_events = disable_outside_pointer_events;

            if let Some(barrier_layer) = entry.pointer_barrier_layer {
                let present = open_now && disable_outside_pointer_events;
                ui.set_layer_visible(barrier_layer, present);
                ui.set_layer_hit_testable(barrier_layer, false);
                ui.set_layer_wants_pointer_down_outside_events(barrier_layer, false);
                ui.set_layer_wants_pointer_move_events(barrier_layer, false);
                ui.set_layer_wants_timer_events(barrier_layer, false);
            }

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
                    on_dismiss_request(
                        &mut host,
                        ActionCx {
                            window,
                            target: trigger,
                        },
                        DismissReason::FocusOutside,
                    );
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
            ui.set_layer_pointer_down_outside_branches(entry.layer, dismissable_branches);
            ui.set_layer_consume_pointer_down_outside_events(
                entry.layer,
                consume_outside_pointer_events && open_now,
            );

            // Non-modal overlays are click-through during close transitions:
            // when `present=true` but `open=false`, they must not participate in hit-testing or
            // the outside-press observer pass.
            OverlayLayer::non_modal_dismissible(true, open_now).apply(ui, entry.layer);

            // Radix-aligned focus restore: when a non-modal overlay closes but remains mounted for
            // a close transition (`present=true`), restore focus deterministically if focus is
            // currently inside the overlay layer (or has been cleared by the layer hide).
            //
            // This mirrors the existing "restore on unmount" policy below, but triggers on the
            // open -> closed edge so recipes can animate out without deferring focus restoration.
            let closing = entry.open && !open_now;
            if closing
                && (consume_outside_pointer_events
                    || focus_scope_prim::should_restore_focus_for_non_modal_overlay(
                        ui,
                        entry.layer,
                    ))
            {
                let focus_in_layer =
                    focus_now.is_some_and(|n| ui.node_layer(n) == Some(entry.layer));
                let focus_cleared_by_modal_scope = modal_barrier_active && focus_now.is_none();
                if (!focus_cleared_by_modal_scope && focus_now.is_none()) || focus_in_layer {
                    if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                        ui,
                        app,
                        window,
                        Some(trigger),
                        entry.restore_focus,
                    ) {
                        ui.set_focus(Some(node));
                    }
                }
            }

            let opening = open_now && (!entry.open || created);
            if opening {
                should_focus_initial = true;
                entry.restore_focus = restore_focus
                    .or_else(|| fret_ui::elements::node_for_element(app, window, trigger));
            }
            entry.open = open_now;
            entry.last_focus = focus_now;
        });

        if should_focus_initial {
            let focus = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.get(&key).and_then(|p| p.initial_focus)
            });
            focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus);
        }
    }

    let to_hide_popovers: Vec<(
        UiLayerId,
        Option<UiLayerId>,
        GlobalElementId,
        bool,
        Option<NodeId>,
    )> = app.with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
        let mut out: Vec<(
            UiLayerId,
            Option<UiLayerId>,
            GlobalElementId,
            bool,
            Option<NodeId>,
        )> = Vec::new();
        for ((w, id), active) in overlays.popovers.iter() {
            if *w != window || seen_popovers.contains(id) {
                continue;
            }
            out.push((
                active.layer,
                active.pointer_barrier_layer,
                active.trigger,
                active.consume_outside_pointer_events,
                active.restore_focus,
            ));
        }
        out
    });

    let to_hide_modals: Vec<(UiLayerId, Option<GlobalElementId>, Option<NodeId>)> = app
        .with_global_mut_untracked(WindowOverlays::default, |overlays, _app| {
            overlays
                .modals
                .iter()
                .filter_map(|((w, id), active)| {
                    if *w != window || seen_modals.contains(id) {
                        return None;
                    }
                    Some((active.layer, active.trigger, active.restore_focus))
                })
                .collect()
        });

    for (layer, pointer_barrier_layer, trigger, consume_outside_pointer_events, restore_focus) in
        to_hide_popovers
    {
        let focus_now = ui.focus();
        let focus_in_layer = focus_now.is_some_and(|n| ui.node_layer(n) == Some(layer));
        let focus_cleared_by_modal_scope = modal_barrier_active && focus_now.is_none();

        // Radix-aligned outcome for menu-like overlays (ADR 0069):
        // when the overlay consumes outside pointer-down events (non-click-through), it's safe to
        // always restore focus to the trigger on unmount (like modals).
        if consume_outside_pointer_events
            || (focus_in_layer
                || (!focus_cleared_by_modal_scope
                    && focus_scope_prim::should_restore_focus_for_non_modal_overlay(ui, layer)))
        {
            OverlayLayer::hide_non_modal_dismissible().apply(ui, layer);
            ui.set_layer_pointer_down_outside_branches(layer, Vec::new());
            ui.set_layer_consume_pointer_down_outside_events(layer, false);
            if let Some(layer) = pointer_barrier_layer {
                ui.set_layer_visible(layer, false);
                ui.set_layer_hit_testable(layer, false);
                ui.set_layer_wants_pointer_down_outside_events(layer, false);
                ui.set_layer_wants_pointer_move_events(layer, false);
                ui.set_layer_wants_timer_events(layer, false);
            }
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
            OverlayLayer::hide_non_modal_dismissible().apply(ui, layer);
            ui.set_layer_pointer_down_outside_branches(layer, Vec::new());
            ui.set_layer_consume_pointer_down_outside_events(layer, false);
            if let Some(layer) = pointer_barrier_layer {
                ui.set_layer_visible(layer, false);
                ui.set_layer_hit_testable(layer, false);
                ui.set_layer_wants_pointer_down_outside_events(layer, false);
                ui.set_layer_wants_pointer_move_events(layer, false);
                ui.set_layer_wants_timer_events(layer, false);
            }
        }
    }

    for (layer, trigger, restore_focus) in to_hide_modals {
        // Modals should restore focus deterministically on close (Radix-style): underlay focus
        // changes cannot happen while the barrier is installed, so it's safe to always restore on
        // unmount.
        OverlayLayer::hide_modal().apply(ui, layer);

        if let Some(node) =
            focus_scope_prim::resolve_restore_focus_node(ui, app, window, trigger, restore_focus)
        {
            ui.set_focus(Some(node));
        }
    }

    for req in hover_overlay_requests {
        if dock_drag_affects_window {
            continue;
        }

        seen_hover_overlays.insert(req.id);

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
                });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            OverlayLayer::hover(true).apply(ui, entry.layer);
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
            OverlayLayer::hide_hover().apply(ui, layer);
            ui.set_focus(Some(trigger_node));
        } else {
            OverlayLayer::hide_hover().apply(ui, layer);
        }
    }

    for req in tooltip_requests {
        seen_tooltips.insert(req.id);

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
                });
            entry.root_name = req.root_name.clone();
            OverlayLayer::tooltip(true).apply(ui, entry.layer);

            ui.set_layer_scroll_dismiss_elements(entry.layer, req.trigger.into_iter().collect());
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
        OverlayLayer::hide_tooltip().apply(ui, layer);
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
                let toast_style = req.style.clone();
                let margin =
                    margin_override.unwrap_or_else(|| theme.metric_required("metric.padding.md"));
                let gap =
                    gap_override.unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
                let toast_padding = theme.metric_required("metric.padding.sm");
                let container_padding = toast_style
                    .container_padding
                    .unwrap_or(fret_core::Edges::all(toast_padding));
                let radius = toast_style
                    .container_radius
                    .unwrap_or_else(|| theme.metric_required("metric.radius.md"));
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
                            let toast_style = toast_style.clone();
                            let store = store_for_toasts.clone();
                            let toast_id = toast.id;
                            let open = toast.open;
                            let position = position;
                            let drag_offset = toast.drag_offset;
                            let settle_from = toast.settle_from;
                            let drag_active = toast.dragging;

                            let fallback_bg = theme
                                .color_by_key("popover")
                                .unwrap_or_else(|| theme.color_required("popover"));
                            let fallback_fg = theme
                                .color_by_key("popover-foreground")
                                .unwrap_or_else(|| theme.color_required("popover-foreground"));

                            let variant_keys = toast_style.palette.for_variant(toast.variant);
                            let bg = theme.color_by_key(&variant_keys.bg).unwrap_or(fallback_bg);
                            let fg = theme.color_by_key(&variant_keys.fg).unwrap_or(fallback_fg);

                            let border_color = toast_style
                                .border_color_key
                                .as_deref()
                                .and_then(|k| theme.color_by_key(k));
                            let fg_muted = toast_style
                                .description
                                .color_key
                                .as_deref()
                                .and_then(|k| theme.color_by_key(k))
                                .or_else(|| {
                                    toast_style
                                        .description_color_key
                                        .as_deref()
                                        .and_then(|k| theme.color_by_key(k))
                                })
                                .unwrap_or_else(|| {
                                    theme.color_by_key("muted-foreground")
                                        .unwrap_or_else(|| theme.color_required("muted-foreground"))
                                });

                            let close = toast.dismissible.then(|| {
                                let close_store = store.clone();
                                let theme = theme.clone();
                                let toast_style = toast_style.clone();
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: true,
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

                                        let state_color = toast_style
                                            .close
                                            .state_layer_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);

                                        let hover_opacity = toast_style
                                            .close
                                            .hover_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.close.hover_state_layer_opacity);
                                        let focus_opacity = toast_style
                                            .close
                                            .focus_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.close.focus_state_layer_opacity);
                                        let pressed_opacity = toast_style
                                            .close
                                            .pressed_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.close.pressed_state_layer_opacity);

                                        let bg = if st.pressed {
                                            Some(alpha_mul(state_color, pressed_opacity))
                                        } else if st.hovered {
                                            Some(alpha_mul(state_color, hover_opacity))
                                        } else if st.focused {
                                            Some(alpha_mul(state_color, focus_opacity))
                                        } else {
                                            None
                                        };

                                        let icon_fg = toast_style
                                            .close
                                            .icon_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: toast_style.close.padding,
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    toast_style.close.radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: "\u{00D7}".into(),
                                                    style: None,
                                                    color: Some(icon_fg),
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
                                let theme = theme.clone();
                                let toast_style = toast_style.clone();
                                let cmd = action.command;
                                let label = action.label;
                                cx.pressable(
                                    fret_ui::element::PressableProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        enabled: true,
                                        focusable: true,
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

                                        let state_color = toast_style
                                            .action
                                            .state_layer_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);

                                        let hover_opacity = toast_style
                                            .action
                                            .hover_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.action.hover_state_layer_opacity);
                                        let focus_opacity = toast_style
                                            .action
                                            .focus_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.action.focus_state_layer_opacity);
                                        let pressed_opacity = toast_style
                                            .action
                                            .pressed_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.action.pressed_state_layer_opacity);

                                        let bg = if st.pressed {
                                            Some(alpha_mul(state_color, pressed_opacity))
                                        } else if st.hovered {
                                            Some(alpha_mul(state_color, hover_opacity))
                                        } else if st.focused {
                                            Some(alpha_mul(state_color, focus_opacity))
                                        } else {
                                            None
                                        };

                                        let label_fg = toast_style
                                            .action
                                            .label_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);
                                        let label_style = toast_style
                                            .action
                                            .label_style_key
                                            .as_deref()
                                            .and_then(|k| theme.text_style_by_key(k));

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: toast_style.action.padding,
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    toast_style.action.radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: label.clone(),
                                                    style: label_style,
                                                    color: Some(label_fg),
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
                                let theme = theme.clone();
                                let toast_style = toast_style.clone();
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

                                        let state_color = toast_style
                                            .cancel
                                            .state_layer_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);

                                        let hover_opacity = toast_style
                                            .cancel
                                            .hover_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.cancel.hover_state_layer_opacity);
                                        let focus_opacity = toast_style
                                            .cancel
                                            .focus_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.cancel.focus_state_layer_opacity);
                                        let pressed_opacity = toast_style
                                            .cancel
                                            .pressed_state_layer_opacity_key
                                            .as_deref()
                                            .and_then(|k| theme.number_by_key(k))
                                            .unwrap_or(toast_style.cancel.pressed_state_layer_opacity);

                                        let bg = if st.pressed {
                                            Some(alpha_mul(state_color, pressed_opacity))
                                        } else if st.hovered {
                                            Some(alpha_mul(state_color, hover_opacity))
                                        } else if st.focused {
                                            Some(alpha_mul(state_color, focus_opacity))
                                        } else {
                                            None
                                        };

                                        let label_fg = toast_style
                                            .cancel
                                            .label_color_key
                                            .as_deref()
                                            .and_then(|k| theme.color_by_key(k))
                                            .unwrap_or(fg);
                                        let label_style = toast_style
                                            .cancel
                                            .label_style_key
                                            .as_deref()
                                            .and_then(|k| theme.text_style_by_key(k));

                                        vec![cx.container(
                                            fret_ui::element::ContainerProps {
                                                layout: fret_ui::element::LayoutStyle::default(),
                                                padding: toast_style.cancel.padding,
                                                background: bg,
                                                shadow: None,
                                                border: fret_core::Edges::all(Px(0.0)),
                                                border_color: None,
                                                focus_ring: None,
                                                focus_border_color: None,
                                                focus_within: false,
                                                corner_radii: fret_core::Corners::all(
                                                    toast_style.cancel.radius,
                                                ),
                                            },
                                            move |cx| {
                                                vec![cx.text_props(fret_ui::element::TextProps {
                                                    layout: fret_ui::element::LayoutStyle::default(
                                                    ),
                                                    text: label.clone(),
                                                    style: label_style,
                                                    color: Some(label_fg),
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

                            let header_theme = theme.clone();
                            let header_style = toast_style.clone();
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
                                                        fret_ui::element::Length::Px(
                                                            header_style.icon_size,
                                                        );
                                                    layout.size.height =
                                                        fret_ui::element::Length::Px(
                                                            header_style.icon_size,
                                                        );
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
                                        style: header_style
                                            .title
                                            .style_key
                                            .as_deref()
                                            .and_then(|k| header_theme.text_style_by_key(k)),
                                        color: Some(
                                            header_style
                                                .title
                                                .color_key
                                                .as_deref()
                                                .and_then(|k| header_theme.color_by_key(k))
                                                .unwrap_or(fg),
                                        ),
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
                                    style: toast_style
                                        .description
                                        .style_key
                                        .as_deref()
                                        .and_then(|k| theme.text_style_by_key(k)),
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
                                toast_layout.size.min_height = if toast.description.is_some() {
                                    toast_style.two_line_min_height
                                } else {
                                    toast_style.single_line_min_height
                                };

                                let toast_el = cx.container(
                                    fret_ui::element::ContainerProps {
                                        layout: toast_layout,
                                        padding: container_padding,
                                        background: Some(bg),
                                        shadow: toast_style.shadow,
                                        border: if border_color.is_some() {
                                            fret_core::Edges::all(toast_style.border_width)
                                        } else {
                                            fret_core::Edges::all(fret_core::Px(0.0))
                                        },
                                        border_color,
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
