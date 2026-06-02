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
mod events;
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
use events::handle_declarative_event;
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
        fret_core::Event::InternalDrag(_)
        | fret_core::Event::Pointer(fret_core::PointerEvent::Down { .. })
        | fret_core::Event::Pointer(fret_core::PointerEvent::Move { .. })
        | fret_core::Event::Pointer(fret_core::PointerEvent::Up { .. })
        | fret_core::Event::Pointer(fret_core::PointerEvent::Wheel { .. })
        | fret_core::Event::PointerCancel(_) => {
            handle_declarative_event(cx, event, window, allow_multi_window_tear_off);
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
