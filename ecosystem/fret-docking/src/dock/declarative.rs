use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, PanelKey, Rect, SemanticsRole, Size};
use fret_ui::element::{AnyElement, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use super::diagnostics::{
    DockingDiagnosticsExtras, diagnostics_env_enabled, publish_docking_diagnostics_snapshot,
    should_publish_docking_diagnostics,
};
use super::host_frame::{DockSpaceLayoutSnapshot, panel_root_placements_for_snapshot};
use super::layout::{dock_space_regions, hidden_bounds};
use super::manager::DockManager;
use super::paint::{
    paint_basic_drop_overlay, paint_complex_drop_overlay_inputs, paint_drag_payload_ghost,
    paint_drop_hints, paint_floating_chrome_inputs, paint_split_handle_inputs,
    paint_tab_chrome_inputs, paint_tab_detail_inputs, paint_tab_insert_preview_title,
    paint_viewport_surface_inputs,
};
use super::services::{DockFocusRequestService, DockPanelContentService};
use super::types::DockDropTarget;
use super::viewport::{
    ViewportCaptureState, viewport_input_from_hit, viewport_input_from_hit_clamped,
};
use fret_runtime::Effect;

mod drag_preview;
mod drag_resolve;
mod drag_route;
mod floating;
mod frame;
mod frame_state;
mod geometry;
mod interaction;
mod overflow;
mod registry;
mod tab_metrics;
mod tab_paint_state;
mod tear_off;

use drag_preview::{
    declarative_tab_insert_preview_title, drag_ghost_title, prepare_declarative_drag_ghost,
};
use drag_resolve::{
    begin_declarative_panel_drag, begin_declarative_tabs_group_drag,
    declarative_panel_drag_allowed, declarative_resolve_internal_drag_drop,
    declarative_resolve_internal_drag_hover, declarative_tabs_group_drag_allowed,
};
use drag_route::{dock_dragging_affects_window, is_dock_drag_kind, keep_internal_drag_route_alive};
use floating::{
    apply_declarative_floating_hover_paint_state, declarative_floating_hover_for_window,
    declarative_hit_test_floating_close, declarative_hit_test_floating_title_bar,
    declarative_resolve_floating_title_bar_drag_target,
};
use frame::DockSpaceElementFrame;
use frame_state::prepare_declarative_frame_paint_state;
use geometry::{
    declarative_hit_test_active_viewport_panel, declarative_hit_test_tab_bar_empty_space,
    declarative_hit_test_tab_close, declarative_hit_test_tab_content,
    declarative_layout_snapshot_for_bounds, declarative_pixels_per_point,
    declarative_split_handle_cursor, declarative_split_handle_hit_for_position,
};
use interaction::{
    DeclarativeDividerDrag, DeclarativeDockInteractionService, DeclarativeFloatingDrag,
    DeclarativeFloatingHover, DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag,
    DeclarativePressedFloatingClose, DeclarativePressedTabClose,
};
use overflow::{
    declarative_handle_tab_overflow_menu_left_click, declarative_handle_tab_overflow_menu_wheel,
    declarative_handle_tab_strip_wheel, declarative_open_tab_overflow_menu,
    declarative_tab_hover_for_position, declarative_tab_overflow_menu_for_window,
};
pub use registry::{
    DockPanelElement, DockPanelElementRegistry, DockPanelElementRegistryService,
    DockSpaceElementOptions, dock_panel_element,
};
use registry::{
    bind_panel_children, collect_panels_for_window, missing_panel_element, panel_nodes_for_window,
};
use tab_metrics::{
    declarative_sync_tab_scroll_for_window, declarative_tab_detail_titles,
    declarative_tab_scroll_for_frame, declarative_tab_widths_for_layout,
    declarative_tab_widths_from_prepared_titles, prepare_declarative_tab_detail_paint,
    prepare_declarative_tab_title,
};
use tab_paint_state::{
    apply_declarative_tab_interaction_paint_state, declarative_tab_hover_for_window,
};
use tear_off::clamp_declarative_floating_rect_to_bounds;

fn publish_declarative_docking_diagnostics<H: UiHost>(app: &mut H, window: AppWindowId) {
    if !should_publish_docking_diagnostics(app, diagnostics_env_enabled()) {
        return;
    }
    let frame_id = app.frame_id();
    publish_docking_diagnostics_snapshot(
        app,
        window,
        frame_id,
        DockingDiagnosticsExtras::default(),
    );
}

fn sync_declarative_viewport_layouts<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    snapshot: &DockSpaceLayoutSnapshot,
) {
    let viewport_layouts = snapshot.viewport_layouts.clone();
    app.with_global_mut_untracked(DockManager::default, |dock, _app| {
        dock.sync_viewport_layouts_for_window(window, viewport_layouts);
    });
}

/// Build a declarative dock-space host from explicit panel roots.
///
/// The host consumes the dock graph and places active panel roots with
/// `DockSpaceLayoutSnapshot`. This is the primary public dock-space entry point when the caller can
/// author panel roots declaratively.
pub fn dock_space_element<H>(
    cx: &mut ElementContext<'_, H>,
    window: AppWindowId,
    options: DockSpaceElementOptions,
    panels: impl IntoIterator<Item = DockPanelElement>,
) -> AnyElement
where
    H: UiHost + 'static,
{
    let panels: Vec<DockPanelElement> = panels.into_iter().collect();
    let panel_keys: Arc<[PanelKey]> = panels
        .iter()
        .map(|panel| panel.panel.clone())
        .collect::<Vec<_>>()
        .into();
    let children: Vec<AnyElement> = panels.into_iter().map(|panel| panel.element).collect();

    let layout_panel_keys = Arc::clone(&panel_keys);
    let layout = options.layout;
    let allow_multi_window_tear_off = options.allow_multi_window_tear_off;
    let mut element = cx.managed_surface_with_prepaint(
        ManagedSurfaceProps {
            layout,
            ..Default::default()
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            let children = cx.children().to_vec();
            bind_panel_children(cx.app(), window, &layout_panel_keys, &children);

            let previous_last_sizes = cx
                .output::<DockSpaceElementFrame>()
                .map(|frame| frame.panel_last_sizes.clone())
                .unwrap_or_default();
            let mut panel_last_sizes = previous_last_sizes;

            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let bounds = cx.bounds();
            let (_chrome, dock_bounds) = dock_space_regions(bounds);
            let snapshot = cx.app().global::<DockManager>().and_then(|dock| {
                DockSpaceLayoutSnapshot::build(
                    dock,
                    window,
                    dock_bounds,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &HashMap::new(),
                )
            });

            let hidden = hidden_bounds(Size::new(fret_core::Px(0.0), fret_core::Px(0.0)));
            let Some(snapshot) = snapshot else {
                for child in children {
                    let _ = cx.layout_child(child, hidden);
                }
                cx.set_output(DockSpaceElementFrame::empty(panel_last_sizes));
                return;
            };

            let theme = cx.theme().snapshot();
            let frame_state = prepare_declarative_frame_paint_state(
                cx.app(),
                window,
                theme,
                &snapshot,
                settings,
                true,
            );
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &frame_state.tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            sync_declarative_viewport_layouts(cx.app(), window, &snapshot);

            let panel_nodes: HashMap<PanelKey, NodeId> = cx
                .app()
                .global::<DockPanelContentService>()
                .map(|content| content.panel_nodes(window).into_iter().collect())
                .unwrap_or_default();
            let mut laid_out = Vec::new();
            for (_panel, node, rect) in
                panel_root_placements_for_snapshot(&snapshot, &panel_nodes, &mut panel_last_sizes)
            {
                let _ = cx.layout_child_root(node, rect);
                laid_out.push(node);
            }

            for child in children {
                if laid_out.contains(&child) {
                    continue;
                }
                let size = panel_keys
                    .iter()
                    .zip(cx.children().iter())
                    .find_map(|(panel, node)| {
                        (*node == child)
                            .then(|| panel_last_sizes.get(panel).copied())
                            .flatten()
                    })
                    .unwrap_or(Size::new(fret_core::Px(0.0), fret_core::Px(0.0)));
                let _ = cx.layout_child(child, hidden_bounds(size));
            }

            cx.set_output(frame_state.into_frame(&snapshot, panel_last_sizes));
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            publish_declarative_docking_diagnostics(cx.app(), window);
            if dock_dragging_affects_window(cx.app(), window) {
                cx.request_animation_frame();
            }

            let settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let bounds = cx.bounds();
            let (_chrome, dock_bounds) = dock_space_regions(bounds);
            let snapshot = cx.app().global::<DockManager>().and_then(|dock| {
                DockSpaceLayoutSnapshot::build(
                    dock,
                    window,
                    dock_bounds,
                    settings.split_handle_gap,
                    settings.split_handle_hit_thickness,
                    &HashMap::new(),
                )
            });

            let Some(snapshot) = snapshot else {
                cx.set_output(DockSpaceElementFrame::empty(HashMap::new()));
                return;
            };

            let theme = cx.theme().snapshot();
            let frame_state = prepare_declarative_frame_paint_state(
                cx.app(),
                window,
                theme,
                &snapshot,
                settings,
                false,
            );
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &frame_state.tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            sync_declarative_viewport_layouts(cx.app(), window, &snapshot);

            let panel_last_sizes = snapshot
                .active_panel_bounds
                .iter()
                .map(|(panel, rect)| (panel.clone(), rect.size))
                .collect();
            cx.set_output(frame_state.into_frame(&snapshot, panel_last_sizes));
        },
        move |cx| {
            let host_node = cx.node();
            keep_internal_drag_route_alive(cx.app(), window, host_node);
            let Some(frame) = cx.output::<DockSpaceElementFrame>().cloned() else {
                return;
            };
            let panel_nodes = panel_nodes_for_window(cx.app(), window);
            let theme = cx.theme().snapshot();
            let scale_factor = cx.scale_factor();
            let overlay_hooks = cx
                .app()
                .global::<super::services::DockViewportOverlayHooksService>()
                .and_then(|svc| svc.hooks());

            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let mut tab_chrome_inputs = frame.tab_chrome_inputs.clone();
            let mut tab_detail_inputs = frame.tab_detail_inputs.clone();
            apply_declarative_tab_interaction_paint_state(
                &frame,
                tab_hover,
                tab_overflow_menu,
                &mut tab_chrome_inputs,
                &mut tab_detail_inputs,
            );
            paint_tab_chrome_inputs(theme.clone(), &tab_chrome_inputs, cx.scene());
            let tab_detail_titles = declarative_tab_detail_titles(cx.app(), &frame);
            let (tab_titles, tab_close_glyph, tab_overflow_glyph) =
                prepare_declarative_tab_detail_paint(
                    tab_detail_titles,
                    cx.services(),
                    scale_factor,
                );
            let measured_tab_widths = declarative_tab_widths_from_prepared_titles(
                cx.app(),
                theme.clone(),
                &frame,
                &tab_titles,
                true,
            );
            cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service.set_tab_widths_for_window(window, measured_tab_widths);
                },
            );
            paint_tab_detail_inputs(
                theme.clone(),
                &tab_detail_inputs,
                &tab_titles,
                Some(tab_close_glyph),
                Some(tab_overflow_glyph),
                None,
                None,
                cx.scene(),
            );
            for title in tab_titles.values() {
                cx.release_text_blob_on_next_paint(title.blob);
            }
            cx.release_text_blob_on_next_paint(tab_close_glyph.blob);
            cx.release_text_blob_on_next_paint(tab_overflow_glyph.blob);

            for (panel, rect) in &frame.paint_panel_bounds {
                if let Some(node) = panel_nodes.get(panel).copied() {
                    let bounds = cx.child_bounds(node).unwrap_or(*rect);
                    cx.paint_child(node, bounds);
                }
            }

            paint_viewport_surface_inputs(
                theme.clone(),
                window,
                &frame.viewport_surface_inputs,
                overlay_hooks.as_deref(),
                cx.scene(),
            );

            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let mut floating_chrome_inputs = frame.floating_chrome_inputs.clone();
            apply_declarative_floating_hover_paint_state(
                &frame,
                floating_hover,
                &mut floating_chrome_inputs,
            );
            paint_floating_chrome_inputs(
                theme.clone(),
                &floating_chrome_inputs,
                None,
                None,
                cx.scene(),
            );

            let drag_ghost = frame.dock_drag_ghost.as_ref().map(|ghost| {
                let title = drag_ghost_title(cx.app(), ghost);
                prepare_declarative_drag_ghost(cx.services(), ghost, &title, scale_factor)
            });
            if let Some(drag_ghost) = drag_ghost.as_ref() {
                paint_drag_payload_ghost(theme.clone(), Some(drag_ghost), false, cx.scene());
            }
            if let Some(drag_ghost) = drag_ghost {
                cx.release_text_blob_on_next_paint(drag_ghost.title.blob);
            }

            paint_basic_drop_overlay(
                theme.clone(),
                frame.hover.clone(),
                window,
                cx.bounds(),
                &frame.layout_all,
                None,
                cx.scene(),
            );
            if let Some((title, drag_source_tabs, tab_count)) =
                declarative_tab_insert_preview_title(cx.app(), window, &frame)
            {
                let prepared = prepare_declarative_tab_title(cx.services(), &title, scale_factor);
                paint_tab_insert_preview_title(
                    theme.clone(),
                    frame.hover.clone(),
                    &frame.layout_all,
                    tab_count,
                    &frame.tab_scroll,
                    &frame.tab_widths,
                    drag_source_tabs,
                    Some(&prepared),
                    false,
                    cx.scene(),
                );
                cx.release_text_blob_on_next_paint(prepared.blob);
            }
            paint_complex_drop_overlay_inputs(
                theme.clone(),
                &frame.complex_drop_overlay_inputs,
                cx.scene(),
            );

            let docking_interaction_settings = cx
                .app()
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let font_size = theme.metric_token("font.size");
            let hint_font_size_inner = fret_core::Px(
                (font_size.0 * docking_interaction_settings.dock_hint_scale_inner.max(0.0))
                    .max(0.0),
            );
            let hint_font_size_outer = fret_core::Px(
                (font_size.0 * docking_interaction_settings.dock_hint_scale_outer.max(0.0))
                    .max(0.0),
            );
            paint_drop_hints(
                theme.clone(),
                frame.drop_hints,
                frame.hover.clone(),
                hint_font_size_inner,
                hint_font_size_outer,
                window,
                cx.bounds(),
                &frame.layout_all,
                cx.scene(),
            );

            paint_split_handle_inputs(
                theme,
                &frame.split_handle_inputs,
                None,
                frame.split_handle_gap,
                frame.split_handle_hit_thickness,
                scale_factor,
                cx.scene(),
            );
        },
        |_cx| children,
    );
    cx.managed_surface_on_command_availability_for(element.id, move |cx, command| {
        if command.as_str() != "dock.focus_requested_panel" {
            return fret_ui::CommandAvailability::NotHandled;
        }

        if cx
            .app()
            .global::<DockFocusRequestService>()
            .is_some_and(|service| service.has(window))
        {
            fret_ui::CommandAvailability::Available
        } else {
            fret_ui::CommandAvailability::NotHandled
        }
    });
    cx.managed_surface_on_command_for(element.id, move |cx, command| {
        if command.as_str() != "dock.focus_requested_panel" {
            return false;
        }

        let Some(panel) = cx.app().with_global_mut(
            DockFocusRequestService::default,
            |service: &mut DockFocusRequestService, _app| service.take(window),
        ) else {
            return false;
        };

        let panel_nodes = panel_nodes_for_window(cx.app(), window);
        if let Some(node) = panel_nodes.get(&panel).copied() {
            cx.request_focus(node);
        } else {
            cx.request_focus(cx.node());
        }
        cx.request_redraw();
        true
    });
    cx.managed_surface_on_event_for(element.id, move |cx, event| match event {
        fret_core::Event::InternalDrag(e)
            if matches!(
                e.kind,
                fret_core::InternalDragKind::Enter | fret_core::InternalDragKind::Over
            ) =>
        {
            let position = cx.pointer_position_window(event).unwrap_or(e.position);
            let bounds = cx.bounds();
            let theme = cx.theme().snapshot();
            let allow_tear_off = cx
                .app()
                .global::<fret_runtime::PlatformCapabilities>()
                .cloned()
                .unwrap_or_default()
                .ui
                .window_tear_off;
            let (effects, changed, invalidate_layout) = declarative_resolve_internal_drag_hover(
                cx.app(),
                window,
                e.pointer_id,
                bounds,
                theme,
                position,
                allow_tear_off,
                allow_multi_window_tear_off,
            );
            for effect in effects {
                cx.push_effect(effect);
            }
            if invalidate_layout {
                cx.invalidate_self(fret_ui::Invalidation::Layout);
            }
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
            }
        }
        fret_core::Event::InternalDrag(e) if e.kind == fret_core::InternalDragKind::Drop => {
            let position = cx.pointer_position_window(event).unwrap_or(e.position);
            let bounds = cx.bounds();
            let theme = cx.theme().snapshot();
            let (effects, changed, invalidate_layout, end_drag) =
                declarative_resolve_internal_drag_drop(
                    cx.app(),
                    window,
                    e.pointer_id,
                    bounds,
                    theme,
                    position,
                    false,
                    false,
                );
            for effect in effects {
                cx.push_effect(effect);
            }
            if invalidate_layout {
                cx.invalidate_self(fret_ui::Invalidation::Layout);
            }
            if end_drag
                && cx
                    .app()
                    .drag(e.pointer_id)
                    .is_some_and(|drag| is_dock_drag_kind(drag.kind))
            {
                cx.app().cancel_drag(e.pointer_id);
            }
            if changed {
                cx.invalidate_self(fret_ui::Invalidation::Paint);
                cx.request_redraw();
            }
        }
        fret_core::Event::InternalDrag(e)
            if matches!(
                e.kind,
                fret_core::InternalDragKind::Leave | fret_core::InternalDragKind::Cancel
            ) =>
        {
            let hover_cleared = cx
                .app()
                .with_global_mut(DockManager::default, |dock, _app| {
                    dock.hover.take().is_some()
                });
            if hover_cleared {
                cx.request_redraw();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position,
            button,
            modifiers,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            if let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            {
                let menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
                if let Some(menu) = menu {
                    let (handled, next_menu, effects) =
                        declarative_handle_tab_overflow_menu_left_click(
                            cx.app(),
                            window,
                            menu,
                            &snapshot.layout_all,
                            theme.clone(),
                            *position,
                        );
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.set_tab_overflow_menu(window, next_menu),
                    );
                    for effect in effects {
                        cx.push_effect(effect);
                    }
                    cx.request_redraw();
                    if handled {
                        cx.stop_propagation();
                        return;
                    }
                } else {
                    let tab_widths = declarative_tab_widths_for_layout(
                        cx.app(),
                        window,
                        theme.clone(),
                        &snapshot.layout_all,
                    );
                    let tab_scroll = declarative_tab_scroll_for_frame(
                        cx.app(),
                        window,
                        theme.clone(),
                        &snapshot.layout_all,
                        &tab_widths,
                        false,
                    );
                    if let Some(menu) = declarative_open_tab_overflow_menu(
                        cx.app(),
                        window,
                        &snapshot.layout_all,
                        &tab_scroll,
                        theme.clone(),
                        *position,
                    ) {
                        cx.app().with_global_mut(
                            DeclarativeDockInteractionService::default,
                            |service, _app| service.set_tab_overflow_menu(window, Some(menu)),
                        );
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                if let Some(floating) = declarative_hit_test_floating_close(&snapshot, *position) {
                    let pressed = DeclarativePressedFloatingClose {
                        pointer_id: *pointer_id,
                        floating,
                    };
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_floating_close(window, pressed),
                    );
                    cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                        window,
                        floating,
                    }));
                    cx.capture_pointer(cx.node());
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
                if let Some((floating, grab_offset, start_rect)) =
                    declarative_hit_test_floating_title_bar(&snapshot, *position)
                    && *button == fret_core::MouseButton::Left
                {
                    let drag = DeclarativeFloatingDrag {
                        pointer_id: *pointer_id,
                        floating,
                        grab_offset,
                        start_rect,
                        start: *position,
                        start_tick: cx.app().tick_id(),
                        activated: false,
                        dock_previews_enabled: false,
                    };
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_floating_drag(window, drag),
                    );
                    cx.push_effect(Effect::Dock(fret_core::DockOp::RaiseFloating {
                        window,
                        floating,
                    }));
                    cx.capture_pointer(cx.node());
                    cx.set_cursor_icon(fret_core::CursorIcon::Default);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if let Some((handle, min_px)) =
                    declarative_split_handle_hit_for_position(cx.app(), &snapshot, *position)
                    && *button == fret_core::MouseButton::Left
                {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| {
                            service.begin_divider_drag(
                                window,
                                DeclarativeDividerDrag {
                                    pointer_id: *pointer_id,
                                    handle,
                                    min_px,
                                },
                            );
                        },
                    );
                    cx.capture_pointer(cx.node());
                    cx.set_cursor_icon(declarative_split_handle_cursor(handle.axis));
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if matches!(
                    *button,
                    fret_core::MouseButton::Left
                        | fret_core::MouseButton::Right
                        | fret_core::MouseButton::Middle
                ) && let Some(hit) =
                    declarative_hit_test_active_viewport_panel(cx.app(), window, bounds, *position)
                {
                    let pixels_per_point = declarative_pixels_per_point(cx.app(), window);
                    if let Some(input) = viewport_input_from_hit(
                        window,
                        hit.clone(),
                        pixels_per_point,
                        *pointer_id,
                        *pointer_type,
                        *position,
                        fret_core::ViewportInputKind::PointerDown {
                            button: *button,
                            modifiers: *modifiers,
                            click_count: *click_count,
                        },
                    ) {
                        cx.push_effect(Effect::ViewportInput(input));
                    }
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| {
                            service.begin_viewport_capture(
                                window,
                                ViewportCaptureState {
                                    pointer_id: *pointer_id,
                                    hit,
                                    button: *button,
                                    start: *position,
                                    last: *position,
                                    moved: false,
                                },
                            );
                        },
                    );
                    cx.capture_pointer(cx.node());
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            if let Some((tabs, index, panel)) =
                declarative_hit_test_tab_close(cx.app(), window, bounds, theme.clone(), *position)
                && *button == fret_core::MouseButton::Left
            {
                let pressed = DeclarativePressedTabClose {
                    pointer_id: *pointer_id,
                    tabs,
                    index,
                    panel,
                    start: *position,
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_tab_close(window, pressed),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            if let Some((_tabs, _index, panel, grab_offset)) =
                declarative_hit_test_tab_content(cx.app(), window, bounds, theme.clone(), *position)
                && *button == fret_core::MouseButton::Left
                && declarative_panel_drag_allowed(cx.app(), window, &panel)
            {
                let pending = DeclarativePendingDockDrag {
                    pointer_id: *pointer_id,
                    start: *position,
                    panel,
                    grab_offset,
                    start_tick: cx.app().tick_id(),
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_pending_dock_drag(window, pending),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            if let Some((tabs, grab_offset)) =
                declarative_hit_test_tab_bar_empty_space(cx.app(), window, bounds, theme, *position)
                && *button == fret_core::MouseButton::Left
                && declarative_tabs_group_drag_allowed(cx.app(), window, tabs)
            {
                let pending = DeclarativePendingDockTabsDrag {
                    pointer_id: *pointer_id,
                    start: *position,
                    tabs,
                    grab_offset,
                    start_tick: cx.app().tick_id(),
                };
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_pending_dock_tabs_drag(window, pending),
                );
                cx.capture_pointer(cx.node());
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position,
            buttons,
            modifiers,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.viewport_capture(window, *pointer_id),
            );
            if let Some(mut capture) = viewport_capture {
                capture.last = *position;
                if !capture.moved && capture.button == fret_core::MouseButton::Right {
                    let settings = cx
                        .app()
                        .global::<fret_runtime::DockingInteractionSettings>()
                        .copied()
                        .unwrap_or_default();
                    let dx = position.x.0 - capture.start.x.0;
                    let dy = position.y.0 - capture.start.y.0;
                    let dist2 = dx * dx + dy * dy;
                    let threshold = settings.viewport_context_menu_drag_threshold.0.max(0.0);
                    capture.moved = dist2 >= threshold * threshold;
                }
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit.clone(),
                    declarative_pixels_per_point(cx.app(), window),
                    *pointer_id,
                    *pointer_type,
                    *position,
                    fret_core::ViewportInputKind::PointerMove {
                        buttons: *buttons,
                        modifiers: *modifiers,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_viewport_capture(window, capture),
                );
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let viewport_capture_exists = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.has_viewport_capture_for_window(window),
            );
            if viewport_capture_exists {
                return;
            }

            let divider_drag = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.divider_drag(window, *pointer_id)
                    } else {
                        service.take_divider_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(divider_drag) = divider_drag {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                cx.set_cursor_icon(declarative_split_handle_cursor(divider_drag.handle.axis));
                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let changed = cx
                    .app()
                    .with_global_mut(DockManager::default, |dock, _app| {
                        let Some((children_len, fractions_now)) = dock
                            .graph
                            .node(divider_drag.handle.split)
                            .and_then(|node| match node {
                                fret_core::DockNode::Split {
                                    children,
                                    fractions,
                                    ..
                                } => Some((children.len(), fractions.clone())),
                                _ => None,
                            })
                        else {
                            return false;
                        };

                        let Some(next) = super::split_geometry::drag_update_adjacent_fractions(
                            divider_drag.handle.axis,
                            divider_drag.handle.bounds,
                            children_len,
                            &fractions_now,
                            divider_drag.handle.handle_ix,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &divider_drag.min_px,
                            divider_drag.handle.grab_offset,
                            *position,
                        ) else {
                            return false;
                        };

                        dock.graph
                            .update_split_fractions(divider_drag.handle.split, next)
                    });
                if changed {
                    cx.invalidate_self(fret_ui::Invalidation::Layout);
                    cx.request_redraw();
                }
                cx.stop_propagation();
                return;
            }

            let drag = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_floating_drag(window, *pointer_id),
            );
            if let Some(mut drag) = drag {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                if !drag.activated {
                    let settings = cx
                        .app()
                        .global::<fret_runtime::DockingInteractionSettings>()
                        .copied()
                        .unwrap_or_default();
                    let activation = fret_dnd::ActivationConstraint::Distance {
                        px: settings.tab_drag_threshold.0,
                    };
                    if activation.is_satisfied(
                        drag.start_tick.0,
                        cx.app().tick_id().0,
                        drag.start,
                        *position,
                    ) {
                        drag.activated = true;
                        drag.dock_previews_enabled =
                            settings.drag_inversion.wants_dock_previews(*modifiers);
                    }
                }

                let desired = Rect::new(
                    fret_core::Point::new(
                        fret_core::Px(position.x.0 - drag.grab_offset.x.0),
                        fret_core::Px(position.y.0 - drag.grab_offset.y.0),
                    ),
                    drag.start_rect.size,
                );
                let rect = clamp_declarative_floating_rect_to_bounds(desired, cx.bounds());
                cx.push_effect(Effect::Dock(fret_core::DockOp::SetFloatingRect {
                    window,
                    floating: drag.floating,
                    rect,
                }));
                if drag.activated {
                    let bounds = cx.bounds();
                    let theme = cx.theme().snapshot();
                    let hover = declarative_resolve_floating_title_bar_drag_target(
                        cx.app(),
                        window,
                        bounds,
                        theme,
                        drag.dock_previews_enabled,
                        *position,
                    );
                    cx.app()
                        .with_global_mut(DockManager::default, |dock, _app| dock.hover = hover);
                }
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.begin_floating_drag(window, drag),
                );
                cx.set_cursor_icon(fret_core::CursorIcon::Default);
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.pending_dock_drag(window, *pointer_id)
                    } else {
                        service.take_pending_dock_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(pending) = pending {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let activation = fret_dnd::ActivationConstraint::Distance {
                    px: settings.tab_drag_threshold.0,
                };
                if activation.is_satisfied(
                    pending.start_tick.0,
                    cx.app().tick_id().0,
                    pending.start,
                    *position,
                ) {
                    let pending = cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.take_pending_dock_drag(window, *pointer_id),
                    );
                    if let Some(pending) = pending {
                        begin_declarative_panel_drag(
                            cx.app(),
                            window,
                            *pointer_id,
                            pending,
                            *position,
                            *modifiers,
                        );
                        cx.app()
                            .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let pending = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    if buttons.left {
                        service.pending_dock_tabs_drag(window, *pointer_id)
                    } else {
                        service.take_pending_dock_tabs_drag(window, *pointer_id)
                    }
                },
            );
            if let Some(pending) = pending {
                if !buttons.left {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }

                let settings = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default();
                let activation = fret_dnd::ActivationConstraint::Distance {
                    px: settings.tab_drag_threshold.0,
                };
                if activation.is_satisfied(
                    pending.start_tick.0,
                    cx.app().tick_id().0,
                    pending.start,
                    *position,
                ) {
                    let pending = cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.take_pending_dock_tabs_drag(window, *pointer_id),
                    );
                    if let Some(pending) = pending {
                        begin_declarative_tabs_group_drag(
                            cx.app(),
                            window,
                            *pointer_id,
                            pending,
                            *position,
                            *modifiers,
                        );
                        cx.app()
                            .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                        cx.release_pointer_capture();
                        cx.request_redraw();
                        cx.stop_propagation();
                        return;
                    }
                }

                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let bounds = cx.bounds();
            let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            else {
                return;
            };
            let split_cursor =
                declarative_split_handle_hit_for_position(cx.app(), &snapshot, *position)
                    .map(|(handle, _min_px)| declarative_split_handle_cursor(handle.axis));
            let floating_close = declarative_hit_test_floating_close(&snapshot, *position);
            let floating_title_bar = declarative_hit_test_floating_title_bar(&snapshot, *position)
                .map(|(floating, _grab_offset, _rect)| floating);
            let floating_hover = DeclarativeFloatingHover {
                close: floating_close,
                title_bar: floating_title_bar,
            };
            let theme = cx.theme().snapshot();
            let (hover, next_menu, pointer_cursor) = declarative_tab_hover_for_position(
                cx.app(),
                window,
                &snapshot.layout_all,
                theme,
                *position,
            );
            let changed = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    let floating_hover_changed = service.set_floating_hover(window, floating_hover);
                    let hover_changed = service.set_tab_hover(window, hover);
                    let menu_changed = !service.tab_overflow_menu_matches(window, &next_menu);
                    service.set_tab_overflow_menu(window, next_menu);
                    floating_hover_changed || hover_changed || menu_changed
                },
            );
            if let Some(cursor) = split_cursor {
                cx.set_cursor_icon(cursor);
            } else if floating_close.is_some() {
                cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
            } else if floating_title_bar.is_some() {
                cx.set_cursor_icon(fret_core::CursorIcon::Default);
            } else if pointer_cursor {
                cx.set_cursor_icon(fret_core::CursorIcon::Pointer);
            }
            if changed {
                cx.request_redraw();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Wheel {
            position, delta, ..
        }) => {
            let bounds = cx.bounds();
            let Some(snapshot) = declarative_layout_snapshot_for_bounds(cx.app(), window, bounds)
            else {
                return;
            };
            let theme = cx.theme().snapshot();
            if let Some(menu) = declarative_tab_overflow_menu_for_window(cx.app(), window) {
                let (handled, next_menu) = declarative_handle_tab_overflow_menu_wheel(
                    cx.app(),
                    menu,
                    &snapshot.layout_all,
                    theme.clone(),
                    *position,
                    *delta,
                );
                if handled {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.set_tab_overflow_menu(window, next_menu),
                    );
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }
            if let Some(next_scroll) = declarative_handle_tab_strip_wheel(
                cx.app(),
                window,
                &snapshot.layout_all,
                theme,
                *position,
                *delta,
            ) {
                declarative_sync_tab_scroll_for_window(
                    cx.app(),
                    window,
                    &next_scroll,
                    snapshot.layout_all.keys().copied(),
                );
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position,
            button,
            modifiers,
            is_click,
            click_count,
            pointer_id,
            pointer_type,
            ..
        }) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_viewport_capture(window, *pointer_id),
            );
            if let Some(capture) = viewport_capture {
                if capture.button != *button {
                    cx.app().with_global_mut(
                        DeclarativeDockInteractionService::default,
                        |service, _app| service.begin_viewport_capture(window, capture),
                    );
                    return;
                }
                let suppress_context_menu = cx
                    .app()
                    .global::<fret_runtime::DockingInteractionSettings>()
                    .copied()
                    .unwrap_or_default()
                    .suppress_context_menu_during_viewport_capture
                    && capture.button == fret_core::MouseButton::Right
                    && capture.moved;
                let is_click = if suppress_context_menu {
                    false
                } else {
                    *is_click
                };
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit.clone(),
                    declarative_pixels_per_point(cx.app(), window),
                    *pointer_id,
                    *pointer_type,
                    *position,
                    fret_core::ViewportInputKind::PointerUp {
                        button: *button,
                        modifiers: *modifiers,
                        is_click,
                        click_count: *click_count,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                if suppress_context_menu {
                    cx.stop_propagation();
                }
                return;
            }

            if *button == fret_core::MouseButton::Left {
                let pressed_floating = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_floating_close(window, *pointer_id),
                );
                if let Some(pressed) = pressed_floating {
                    let bounds = cx.bounds();
                    let clicked =
                        declarative_layout_snapshot_for_bounds(cx.app(), window, bounds).and_then(
                            |snapshot| declarative_hit_test_floating_close(&snapshot, *position),
                        ) == Some(pressed.floating);
                    if clicked
                        && let Some(target_tabs) = cx
                            .app()
                            .global::<DockManager>()
                            .and_then(|dock| dock.graph.first_tabs_in_window(window))
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: pressed.floating,
                            target_tabs,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let floating_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_floating_drag(window, *pointer_id),
                );
                if let Some(drag) = floating_drag {
                    let bounds = cx.bounds();
                    let theme = cx.theme().snapshot();
                    if drag.activated
                        && drag.dock_previews_enabled
                        && let Some(DockDropTarget::Dock(target)) =
                            declarative_resolve_floating_title_bar_drag_target(
                                cx.app(),
                                window,
                                bounds,
                                theme,
                                drag.dock_previews_enabled,
                                *position,
                            )
                        && matches!(target.zone, fret_core::DropZone::Center)
                    {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::MergeFloatingInto {
                            window,
                            floating: drag.floating,
                            target_tabs: target.tabs,
                        }));
                    }
                    cx.app()
                        .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let divider_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_divider_drag(window, *pointer_id),
                );
                if let Some(divider_drag) = divider_drag {
                    let updates = cx
                        .app()
                        .with_global_mut(DockManager::default, |dock, _app| {
                            dock.graph
                                .node(divider_drag.handle.split)
                                .and_then(|node| match node {
                                    fret_core::DockNode::Split {
                                        children,
                                        fractions,
                                        ..
                                    } if children.len() >= 2
                                        && children.len() == fractions.len() =>
                                    {
                                        Some(vec![fret_core::SplitFractionsUpdate {
                                            split: divider_drag.handle.split,
                                            fractions: fractions.clone(),
                                        }])
                                    }
                                    _ => None,
                                })
                                .unwrap_or_default()
                        });
                    if !updates.is_empty() {
                        cx.push_effect(Effect::Dock(fret_core::DockOp::SetSplitFractionsMany {
                            updates,
                        }));
                    }
                    cx.release_pointer_capture();
                    cx.invalidate_self(fret_ui::Invalidation::Layout);
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_pending_dock_drag(window, *pointer_id),
                );
                if pending_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            if *button == fret_core::MouseButton::Left {
                let pending_tabs_drag = cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_pending_dock_tabs_drag(window, *pointer_id),
                );
                if pending_tabs_drag.is_some() {
                    cx.release_pointer_capture();
                    cx.request_redraw();
                    cx.stop_propagation();
                    return;
                }
            }

            let pressed = if *button == fret_core::MouseButton::Left {
                cx.app().with_global_mut(
                    DeclarativeDockInteractionService::default,
                    |service, _app| service.take_tab_close(window, *pointer_id),
                )
            } else {
                None
            };
            let Some(pressed) = pressed else {
                return;
            };
            let theme = cx.theme().snapshot();
            let bounds = cx.bounds();
            let clicked =
                declarative_hit_test_tab_close(cx.app(), window, bounds, theme, *position)
                    .is_some_and(|(tabs, index, panel)| {
                        tabs == pressed.tabs && index == pressed.index && panel == pressed.panel
                    });
            let within_slop = fret_ui_headless::tab_strip_hit_test::pointer_move_within_slop(
                pressed.start,
                *position,
                super::consts::DOCK_TAB_CLOSE_CLICK_SLOP,
            );
            if clicked || within_slop {
                cx.push_effect(Effect::Dock(fret_core::DockOp::ClosePanel {
                    window,
                    panel: pressed.panel,
                }));
            }
            cx.release_pointer_capture();
            cx.request_redraw();
            cx.stop_propagation();
        }
        fret_core::Event::PointerCancel(cancel) => {
            let viewport_capture = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.take_viewport_capture(window, cancel.pointer_id),
            );
            if let Some(capture) = viewport_capture {
                let position = cancel.position.unwrap_or(capture.last);
                let input = viewport_input_from_hit_clamped(
                    window,
                    capture.hit,
                    declarative_pixels_per_point(cx.app(), window),
                    cancel.pointer_id,
                    cancel.pointer_type,
                    position,
                    fret_core::ViewportInputKind::PointerCancel {
                        buttons: cancel.buttons,
                        modifiers: cancel.modifiers,
                        reason: cancel.reason,
                    },
                );
                cx.push_effect(Effect::ViewportInput(input));
                cx.app()
                    .with_global_mut(DockManager::default, |dock, _app| dock.hover = None);
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
                return;
            }

            let cleared_tab = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service.take_tab_close(window, cancel.pointer_id).is_some()
                        || service
                            .take_pending_dock_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_pending_dock_tabs_drag(window, cancel.pointer_id)
                            .is_some()
                        || service
                            .take_divider_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            let cleared_floating = cx.app().with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| {
                    service
                        .take_floating_close(window, cancel.pointer_id)
                        .is_some()
                        || service
                            .take_floating_drag(window, cancel.pointer_id)
                            .is_some()
                },
            );
            if cleared_tab || cleared_floating {
                cx.release_pointer_capture();
                cx.request_redraw();
                cx.stop_propagation();
            }
        }
        _ => {}
    });

    let mut semantics = fret_ui::element::SemanticsDecoration::default().role(SemanticsRole::Panel);
    if let Some(test_id) = options.test_id {
        semantics = semantics.test_id(test_id);
    }
    element = element.attach_semantics(semantics);
    element
}

/// Build a declarative dock-space host from the installed declarative panel registry.
///
/// Non-viewport panels without a registered declarative element receive the same generic missing-UI
/// placeholder as the retained registry path. Pure viewport panels may omit an element root.
pub fn dock_space_element_from_registry<H>(
    cx: &mut ElementContext<'_, H>,
    window: AppWindowId,
    options: DockSpaceElementOptions,
) -> AnyElement
where
    H: UiHost + 'static,
{
    let registry = cx
        .app
        .global::<DockPanelElementRegistryService<H>>()
        .and_then(|service| service.registry());
    let panels = collect_panels_for_window(cx.app, window);
    let mut elements = Vec::new();

    for (panel, is_viewport_panel) in panels {
        let element = registry
            .as_ref()
            .and_then(|registry| registry.render_panel(cx, window, &panel));
        let element = match (is_viewport_panel, element) {
            (_, Some(element)) => element,
            (true, None) => continue,
            (false, None) => missing_panel_element(cx, &panel),
        };
        elements.push(DockPanelElement::new(panel, element));
    }

    dock_space_element(cx, window, options, elements)
}

/// Mount a declarative dock-space host into an immediate-style writer.
#[cfg(feature = "imui")]
pub fn imui_dock_space_element<H: UiHost + 'static>(
    ui: &mut impl fret_authoring::UiWriter<H>,
    options: DockSpaceElementOptions,
) {
    let window = ui.with_cx_mut(|cx| cx.window);
    ui.mount(|cx| vec![dock_space_element_from_registry(cx, window, options)]);
}
