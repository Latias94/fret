use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{AppWindowId, Color, NodeId, Point, Px, Rect, Transform2D};
use fret_runtime::DragKind;
use fret_ui::action::{DismissReason, UiActionHostExt};
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
use super::{ToastPosition, ToastVariant, dismiss_toast_action};

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
    let dock_drag_affects_window = app.drag().is_some_and(|d| {
        d.kind == DragKind::DockPanel && (d.source_window == window || d.current_window == window)
    });

    let (
        modal_requests,
        popover_requests,
        hover_overlay_requests,
        tooltip_requests,
        toast_requests,
    ) = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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

        let open_now = app.models().get_copied(&req.open).unwrap_or(false);

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |cx| {
                let open = req.open;
                cx.dismissible_on_dismiss_request(Arc::new(
                    move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                    },
                ));
                req.children
            },
        );

        let key = (window, req.id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        app.with_global_mut(WindowOverlays::default, |overlays, app| {
            let mut created = false;
            let entry = overlays.modals.entry(key).or_insert_with(|| {
                created = true;
                ActiveModal {
                    layer: ui.push_overlay_root_ex(root, true, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    initial_focus: req.initial_focus,
                    open: false,
                    restore_focus: None,
                }
            });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;

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
            }
            entry.open = open_now;
        });

        if should_focus_initial {
            let focus = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
                overlays.modals.get(&key).and_then(|p| p.initial_focus)
            });
            focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus);
        }
    }

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
        let on_pointer_move = req.on_pointer_move.clone();

        let root = declarative::render_dismissible_root_with_hooks(
            ui,
            app,
            services,
            window,
            bounds,
            &req.root_name,
            |cx| {
                let open = req.open.clone();
                if let Some(on_pointer_move) = on_pointer_move {
                    cx.dismissible_on_pointer_move(on_pointer_move);
                }
                cx.dismissible_on_dismiss_request(Arc::new(
                    move |host, _cx, _reason: DismissReason| {
                        let _ = host.models_mut().update(&open, |v| *v = false);
                    },
                ));
                req.children
            },
        );

        let key = (window, req.id);
        let restore_focus = ui.focus();

        let mut should_focus_initial = false;
        app.with_global_mut(WindowOverlays::default, |overlays, app| {
            let mut created = false;
            let entry = overlays.popovers.entry(key).or_insert_with(|| {
                created = true;
                ActivePopover {
                    layer: ui.push_overlay_root_ex(root, false, true),
                    root_name: req.root_name.clone(),
                    trigger: req.trigger,
                    initial_focus: req.initial_focus,
                    consume_outside_pointer_events: req.consume_outside_pointer_events,
                    open: false,
                    restore_focus: None,
                    last_focus: focus_now,
                }
            });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;
            entry.consume_outside_pointer_events = req.consume_outside_pointer_events;

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
                let _ = app.models_mut().update(&req.open, |v| *v = false);
                open_now = false;
            }

            let dismissable_branches = if open_now {
                dismissable_branch_nodes.clone()
            } else {
                Vec::new()
            };
            ui.set_layer_pointer_down_outside_branches(entry.layer, dismissable_branches);
            ui.set_layer_consume_pointer_down_outside_events(
                entry.layer,
                req.consume_outside_pointer_events && open_now,
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
                && (req.consume_outside_pointer_events
                    || focus_scope_prim::should_restore_focus_for_non_modal_overlay(
                        ui,
                        entry.layer,
                    ))
            {
                let focus_in_layer =
                    focus_now.is_some_and(|n| ui.node_layer(n) == Some(entry.layer));
                if focus_now.is_none() || focus_in_layer {
                    if let Some(node) = focus_scope_prim::resolve_restore_focus_node(
                        ui,
                        app,
                        window,
                        Some(req.trigger),
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
            }
            entry.open = open_now;
            entry.last_focus = focus_now;
        });

        if should_focus_initial {
            let focus = app.with_global_mut(WindowOverlays::default, |overlays, _app| {
                overlays.popovers.get(&key).and_then(|p| p.initial_focus)
            });
            focus_scope_prim::apply_initial_focus_for_overlay(ui, app, window, root, focus);
        }
    }

    let to_hide_popovers: Vec<(UiLayerId, GlobalElementId, bool, Option<NodeId>)> = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
            let mut out: Vec<(UiLayerId, GlobalElementId, bool, Option<NodeId>)> = Vec::new();
            for ((w, id), active) in overlays.popovers.iter() {
                if *w != window || seen_popovers.contains(id) {
                    continue;
                }
                out.push((
                    active.layer,
                    active.trigger,
                    active.consume_outside_pointer_events,
                    active.restore_focus,
                ));
            }
            out
        });

    let to_hide_modals: Vec<(UiLayerId, Option<GlobalElementId>, Option<NodeId>)> = app
        .with_global_mut(WindowOverlays::default, |overlays, _app| {
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

    for (layer, trigger, consume_outside_pointer_events, restore_focus) in to_hide_popovers {
        // Radix-aligned outcome for menu-like overlays (ADR 0069):
        // when the overlay consumes outside pointer-down events (non-click-through), it's safe to
        // always restore focus to the trigger on unmount (like modals).
        if consume_outside_pointer_events
            || focus_scope_prim::should_restore_focus_for_non_modal_overlay(ui, layer)
        {
            OverlayLayer::hide_non_modal_dismissible().apply(ui, layer);
            ui.set_layer_pointer_down_outside_branches(layer, Vec::new());
            ui.set_layer_consume_pointer_down_outside_events(layer, false);
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
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
                if let Some(on_pointer_move) = on_pointer_move {
                    cx.dismissible_on_pointer_move(on_pointer_move);
                }
                children
            },
        );

        let key = (window, req.id);
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let entry = overlays
                .tooltips
                .entry(key)
                .or_insert_with(|| ActiveTooltip {
                    layer: ui.push_overlay_root_ex(root, false, false),
                    root_name: req.root_name.clone(),
                });
            entry.root_name = req.root_name.clone();
            OverlayLayer::tooltip(true).apply(ui, entry.layer);
        });
    }

    let to_hide_tooltips: Vec<UiLayerId> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
                let margin = margin_override.unwrap_or_else(|| theme.metric_required("metric.padding.md"));
                let gap = gap_override.unwrap_or_else(|| theme.metric_required("metric.padding.sm"));
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
                        for toast in toasts {
                            let store = store_for_toasts.clone();
                            let toast_id = toast.id;
                            let open = toast.open;
                            let position = position;
                            let drag_x = toast.drag_x;
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
                                    gap: theme.metric_required("metric.padding.sm"),
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
                                                gap: theme.metric_required("metric.padding.sm"),
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
                                let slide =
                                    Transform2D::translation(Point::new(Px(dx.0 + drag_x.0), dy));

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
                                                            if end.dx.0.abs() >= 80.0 {
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
                                        vec![cx.visual_transform_props(
                                            fret_ui::element::VisualTransformProps {
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
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
