use std::collections::HashSet;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, Rect};
use fret_runtime::DragKind;
use fret_ui::action::{DismissReason, UiActionHostExt};
use fret_ui::declarative;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{Invalidation, UiHost, UiTree};

use crate::primitives::focus_scope as focus_scope_prim;

use super::state::{
    ActiveHoverOverlay, ActiveModal, ActivePopover, ActiveToastLayer, ActiveTooltip, OverlayLayer,
    WindowOverlays,
};
use super::toast::ToastEntry;
use super::{ToastPosition, ToastVariant, dismiss_toast_action};

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
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
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
        let dismissable_branch_nodes: Vec<NodeId> = req
            .dismissable_branches
            .iter()
            .filter_map(|branch| fret_ui::elements::node_for_element(app, window, *branch))
            .collect();

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
                    open: false,
                    restore_focus: None,
                    last_focus: focus_now,
                }
            });
            entry.root_name = req.root_name.clone();
            entry.trigger = req.trigger;
            entry.initial_focus = req.initial_focus;

            if open_now
                && let Some(focus) = focus_now
                && entry.last_focus != Some(focus)
                && let Some(layer_root) = ui.layer_root(entry.layer)
            {
                let focus_inside = ui.is_descendant(layer_root, focus)
                    || dismissable_branch_nodes
                        .iter()
                        .copied()
                        .any(|branch| ui.is_descendant(branch, focus));

                if !focus_inside {
                    let _ = app.models_mut().update(&req.open, |v| *v = false);
                    open_now = false;
                }
            }

            ui.set_layer_pointer_down_outside_branches(
                entry.layer,
                open_now
                    .then(|| dismissable_branch_nodes.clone())
                    .unwrap_or_default(),
            );

            // Non-modal overlays are click-through during close transitions:
            // when `present=true` but `open=false`, they must not participate in hit-testing or
            // the outside-press observer pass.
            OverlayLayer::non_modal_dismissible(true, open_now).apply(ui, entry.layer);

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

    let to_hide_popovers: Vec<(UiLayerId, GlobalElementId, Option<NodeId>)> =
        app.with_global_mut(WindowOverlays::default, |overlays, _app| {
            let mut out: Vec<(UiLayerId, GlobalElementId, Option<NodeId>)> = Vec::new();
            for ((w, id), active) in overlays.popovers.iter() {
                if *w != window || seen_popovers.contains(id) {
                    continue;
                }
                out.push((active.layer, active.trigger, active.restore_focus));
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

    for (layer, trigger, restore_focus) in to_hide_popovers {
        if focus_scope_prim::should_restore_focus_for_non_modal_overlay(ui, layer) {
            OverlayLayer::hide_non_modal_dismissible().apply(ui, layer);
            ui.set_layer_pointer_down_outside_branches(layer, Vec::new());
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
                        host.update_weak_model(&hook_store, |st| st.remove_toast_by_token(token))
                            .flatten()
                            .is_some()
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

                let theme = fret_ui::Theme::global(&*cx.app).clone();
                let margin = theme.metrics.padding_md;
                let gap = theme.metrics.padding_sm;
                let toast_padding = theme.metrics.padding_sm;
                let radius = theme.metrics.radius_md;
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
                    ToastPosition::TopRight => {
                        wrapper_layout.inset.top = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                    ToastPosition::BottomLeft => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.left = Some(margin);
                    }
                    ToastPosition::BottomRight => {
                        wrapper_layout.inset.bottom = Some(margin);
                        wrapper_layout.inset.right = Some(margin);
                    }
                }

                vec![cx.flex(
                    fret_ui::element::FlexProps {
                        layout: wrapper_layout,
                        direction: fret_core::Axis::Vertical,
                        gap,
                        padding: fret_core::Edges::all(fret_core::Px(0.0)),
                        justify: fret_ui::element::MainAlign::End,
                        align: fret_ui::element::CrossAlign::End,
                        wrap: false,
                    },
                    move |cx| {
                        let mut out: Vec<AnyElement> = Vec::with_capacity(toasts.len());
                        for toast in toasts {
                            let store = store_for_toasts.clone();
                            let toast_id = toast.id;

                            let bg = match toast.variant {
                                ToastVariant::Default => theme.colors.panel_background,
                                ToastVariant::Destructive => theme.colors.menu_background,
                            };
                            let border_color = theme.colors.panel_border;
                            let fg = theme.colors.text_primary;
                            let fg_muted = theme.colors.text_muted;

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
                                    move |cx, _st| {
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
                                        vec![cx.text("×")]
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
                                    move |cx, _st| {
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
                                        vec![cx.text(label.as_ref())]
                                    },
                                )
                            });

                            let header_row = cx.flex(
                                fret_ui::element::FlexProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    direction: fret_core::Axis::Horizontal,
                                    gap: theme.metrics.padding_sm,
                                    padding: fret_core::Edges::all(fret_core::Px(0.0)),
                                    justify: fret_ui::element::MainAlign::Start,
                                    align: fret_ui::element::CrossAlign::Center,
                                    wrap: false,
                                },
                                |cx| {
                                    let mut row: Vec<AnyElement> = Vec::new();
                                    row.push(cx.text_props(fret_ui::element::TextProps {
                                        layout: fret_ui::element::LayoutStyle::default(),
                                        text: toast.title.clone(),
                                        style: None,
                                        color: Some(fg),
                                        wrap: fret_core::TextWrap::None,
                                        overflow: fret_core::TextOverflow::Clip,
                                    }));
                                    if let Some(action) = action {
                                        row.push(action);
                                    }
                                    if let Some(close) = close {
                                        row.push(close);
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

                            out.push(cx.container(
                                fret_ui::element::ContainerProps {
                                    layout: fret_ui::element::LayoutStyle::default(),
                                    padding: fret_core::Edges::all(toast_padding),
                                    background: Some(bg),
                                    shadow: None,
                                    border: fret_core::Edges::all(fret_core::Px(1.0)),
                                    border_color: Some(border_color),
                                    corner_radii: fret_core::Corners::all(radius),
                                },
                                move |_cx| toast_children,
                            ));
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
