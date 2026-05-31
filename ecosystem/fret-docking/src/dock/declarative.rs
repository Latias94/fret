use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, NodeId, PanelKey, Rect, SemanticsRole, Size};
use fret_ui::element::{AnyElement, ManagedSurfaceProps};
use fret_ui::{ElementContext, UiHost};

use super::diagnostics::{
    DockingDiagnosticsExtras, diagnostics_env_enabled, publish_docking_diagnostics_snapshot,
    should_publish_docking_diagnostics,
};
use super::drop_resolve::{
    DockPanelDropDrag, DockTabsDropDrag, apply_dock_drop_intent,
    compute_dock_drop_resolve_diagnostics, dock_drop_intent_debug_kind,
    dock_drop_target_diagnostics, resolve_dock_drop_intent_panel, resolve_dock_drop_intent_tabs,
    resolve_dock_drop_target,
};
use super::host_frame::{
    DockSpaceLayoutSnapshot, begin_cross_window_dock_drag, panel_root_placements_for_snapshot,
};
use super::layout::{dock_space_regions, hidden_bounds};
use super::manager::DockManager;
use super::paint::{
    TabChromePaintInput, TabDetailPaintInput, complex_drop_overlay_paint_inputs,
    paint_basic_drop_overlay, paint_complex_drop_overlay_inputs, paint_drag_payload_ghost,
    paint_drop_hints, paint_floating_chrome_inputs, paint_split_handle_inputs,
    paint_tab_chrome_inputs, paint_tab_detail_inputs, paint_tab_insert_preview_title,
    paint_viewport_surface_inputs, split_handle_paint_inputs, tab_chrome_paint_inputs,
    tab_detail_paint_inputs, viewport_surface_paint_inputs,
};
use super::services::{DockFocusRequestService, DockPanelContentService, DockingPolicyService};
use super::tab_overflow::TabOverflowMenuState;
use super::types::{DockDropHints, DockDropTarget, DockPanelDragPayload, DockTabsDragPayload};
use super::viewport::{
    ViewportCaptureState, viewport_input_from_hit, viewport_input_from_hit_clamped,
};
use fret_runtime::Effect;

mod drag_preview;
mod floating;
mod frame;
mod geometry;
mod interaction;
mod overflow;
mod registry;
mod tab_metrics;
mod tear_off;

use drag_preview::{
    declarative_tab_insert_preview_title, dock_drag_ghost_for_window, drag_ghost_title,
    prepare_declarative_drag_ghost,
};
use floating::{
    apply_declarative_floating_hover_paint_state, declarative_floating_hover_for_window,
    declarative_hit_test_floating_close, declarative_hit_test_floating_title_bar,
    declarative_pressed_floating_close_for_window,
    declarative_resolve_floating_title_bar_drag_target, floating_chrome_paint_inputs,
};
use frame::DockSpaceElementFrame;
use geometry::{
    declarative_hit_test_active_viewport_panel, declarative_hit_test_tab_bar_empty_space,
    declarative_hit_test_tab_close, declarative_hit_test_tab_content,
    declarative_layout_snapshot_for_bounds, declarative_pixels_per_point,
    declarative_split_handle_cursor, declarative_split_handle_hit_for_position,
};
use interaction::{
    DeclarativeDividerDrag, DeclarativeDockInteractionService, DeclarativeFloatingDrag,
    DeclarativeFloatingHover, DeclarativePendingDockDrag, DeclarativePendingDockTabsDrag,
    DeclarativePressedFloatingClose, DeclarativePressedTabClose, DeclarativeTabHover,
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
    declarative_apply_tab_bar_drag_auto_scroll, declarative_sync_tab_scroll_for_window,
    declarative_tab_detail_titles, declarative_tab_scroll_for_frame,
    declarative_tab_widths_for_layout, declarative_tab_widths_from_prepared_titles,
    prepare_declarative_tab_detail_paint, prepare_declarative_tab_title,
};
use tear_off::{
    clamp_declarative_floating_rect_to_bounds, declarative_allow_tear_off_for_panel,
    declarative_default_floating_rect_for_panel, declarative_resolve_tear_off_hover,
};

fn keep_internal_drag_route_alive<H: UiHost>(app: &mut H, window: AppWindowId, host_node: NodeId) {
    fret_ui::internal_drag::set_route(app, window, fret_runtime::DRAG_KIND_DOCK_PANEL, host_node);
    fret_ui::internal_drag::set_route(app, window, fret_runtime::DRAG_KIND_DOCK_TABS, host_node);
    if app.global::<DockManager>().is_some() {
        app.with_global_mut_untracked(DockManager::default, |dock, _app| {
            dock.register_dock_space_node(window, host_node);
        });
    }
}

fn dock_dragging_affects_window<H: UiHost>(app: &H, window: AppWindowId) -> bool {
    app.any_drag_session(|drag| {
        (drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && (drag.source_window == window || drag.current_window == window)
            && drag.dragging
    })
}

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

fn declarative_tab_hover_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> DeclarativeTabHover {
    app.global::<DeclarativeDockInteractionService>()
        .map(|service| service.tab_hover(window))
        .unwrap_or_default()
}

fn declarative_dragged_tab_for_drop<H: UiHost>(
    app: &H,
    drag: &fret_runtime::DragSession,
) -> Option<(fret_core::DockNodeId, usize)> {
    let payload = drag.payload::<DockPanelDragPayload>()?;
    app.global::<DockManager>()?
        .graph
        .find_panel_in_window(drag.source_window, &payload.panel)
}

#[allow(clippy::too_many_arguments)]
fn declarative_resolve_internal_drag_drop<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> (Vec<Effect>, bool, bool, bool) {
    let Some(drag) = app.drag(pointer_id) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, false);
    };

    let dock_previews_enabled = drag
        .payload::<DockPanelDragPayload>()
        .map(|payload| payload.dock_previews_enabled)
        .or_else(|| {
            drag.payload::<DockTabsDragPayload>()
                .map(|payload| payload.dock_previews_enabled)
        })
        .unwrap_or(false);
    let dragging = drag.dragging || drag.source_window != window;
    if !dragging {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    }

    let source_window = drag.source_window;
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();

    let Some(snapshot) = declarative_layout_snapshot_for_bounds(app, window, bounds) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    };
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let font_size = theme.metric_token("font.size");
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let prev_hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.hover.clone());
    let mut candidates = Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
    let graph = &app.global::<DockManager>().expect("dock manager").graph;
    let (target, source) = resolve_dock_drop_target(
        prev_hover,
        !dock_previews_enabled,
        true,
        window,
        policy.as_deref(),
        graph,
        snapshot.root,
        dock_bounds,
        bounds,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        hint_font_size_inner,
        hint_font_size_outer,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        dragged_tab_for_drop,
        diagnostics_enabled.then_some(&mut candidates),
    );

    let panel_last_sizes: HashMap<PanelKey, Size> = snapshot
        .paint_panel_bounds
        .iter()
        .map(|(panel, rect)| (panel.clone(), rect.size))
        .collect();
    let mut effects = Vec::new();
    let mut invalidate_layout = false;
    let intent = if let Some(payload) = panel_payload.as_ref() {
        let allow_panel_tear_off = declarative_allow_tear_off_for_panel(
            app,
            allow_tear_off,
            allow_multi_window_tear_off,
            source_window,
            &payload.panel,
        );
        resolve_dock_drop_intent_panel(
            target.clone(),
            DockPanelDropDrag {
                source_window,
                panel: &payload.panel,
                grab_offset: payload.grab_offset,
                tear_off_requested: payload.tear_off_requested,
            },
            window,
            bounds,
            position,
            allow_panel_tear_off,
            false,
            |panel, position, grab_offset, window_bounds| {
                declarative_default_floating_rect_for_panel(
                    panel,
                    position,
                    grab_offset,
                    window_bounds,
                    &panel_last_sizes,
                )
            },
        )
    } else if let Some(payload) = tabs_payload.as_ref() {
        let panel = payload
            .tabs
            .get(payload.active)
            .or_else(|| payload.tabs.first());
        let allow_tabs_tear_off = panel.is_some_and(|panel| {
            declarative_allow_tear_off_for_panel(
                app,
                allow_tear_off,
                allow_multi_window_tear_off,
                source_window,
                panel,
            )
        });
        resolve_dock_drop_intent_tabs(
            target.clone(),
            DockTabsDropDrag {
                source_window,
                source_tabs: payload.source_tabs,
                tabs: &payload.tabs,
                active: payload.active,
                grab_offset: payload.grab_offset,
                tear_off_requested: payload.tear_off_requested,
            },
            window,
            bounds,
            position,
            allow_tabs_tear_off,
            false,
            |panel, position, grab_offset, window_bounds| {
                declarative_default_floating_rect_for_panel(
                    panel,
                    position,
                    grab_offset,
                    window_bounds,
                    &panel_last_sizes,
                )
            },
        )
    } else {
        super::types::DockDropIntent::None
    };
    apply_dock_drop_intent(intent.clone(), &mut effects, &mut invalidate_layout);

    let (graph_stats, graph_signature, diagnostics) =
        app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover = None;
            let graph_stats = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_stats_for_window(&dock.graph, window));
            let graph_signature = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_signature_for_window(&dock.graph, window));
            let diagnostics = diagnostics_enabled.then(|| {
                compute_dock_drop_resolve_diagnostics(
                    pointer_id,
                    position,
                    bounds,
                    dock_bounds,
                    source,
                    &dock.graph,
                    window,
                    target.as_ref(),
                    candidates,
                )
            });
            (graph_stats, graph_signature, diagnostics)
        });

    let frame_id = app.frame_id();
    if let Some(dock_drop_resolve) = diagnostics {
        app.with_global_mut_untracked(
            fret_runtime::WindowInteractionDiagnosticsStore::default,
            |svc, _app| {
                svc.record_docking(
                    window,
                    frame_id,
                    fret_runtime::DockingInteractionDiagnostics {
                        dock_drop_resolve: Some(dock_drop_resolve),
                        dock_graph_stats: graph_stats,
                        dock_graph_signature: graph_signature,
                        ..Default::default()
                    },
                );
            },
        );
    }
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) {
        let drop_target_diag = dock_drop_target_diagnostics(target.as_ref());
        tracing::info!(
            window = ?window,
            source_window = ?source_window,
            pointer_id = ?pointer_id,
            pos = ?position,
            invert_docking = !dock_previews_enabled,
            resolve_source = ?source,
            drop_target = ?drop_target_diag,
            intent_kind = dock_drop_intent_debug_kind(&intent),
            "declarative dock drag drop"
        );
    }

    (effects, true, invalidate_layout, true)
}

fn declarative_resolve_internal_drag_hover<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    bounds: Rect,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
) -> (Vec<Effect>, bool, bool) {
    let Some(drag) = app.drag(pointer_id) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false);
    };
    let dock_previews_enabled = drag
        .payload::<DockPanelDragPayload>()
        .map(|payload| payload.dock_previews_enabled)
        .or_else(|| {
            drag.payload::<DockTabsDragPayload>()
                .map(|payload| payload.dock_previews_enabled)
        })
        .unwrap_or(false);
    let dragging = drag.dragging || drag.source_window != window;
    if !dragging {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false);
    }
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);

    let Some(snapshot) = declarative_layout_snapshot_for_bounds(app, window, bounds) else {
        return (Vec::new(), false, false);
    };
    let tear_off = declarative_resolve_tear_off_hover(
        app,
        window,
        pointer_id,
        bounds,
        position,
        allow_tear_off,
        allow_multi_window_tear_off,
    );
    if tear_off.requested_tear_off {
        let _hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.hover.take().is_some()
        });
        return (tear_off.effects, true, true);
    }
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let font_size = theme.metric_token("font.size");
    let hint_font_size_inner =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer =
        fret_core::Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));
    let tab_widths =
        declarative_tab_widths_for_layout(app, window, theme.clone(), &snapshot.layout_all);
    let mut tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        &snapshot.layout_all,
        &tab_widths,
        false,
    );
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let mut candidates = Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
    let (mut hover, source) = resolve_dock_drop_target(
        None,
        !dock_previews_enabled,
        true,
        window,
        policy.as_deref(),
        &app.global::<DockManager>().expect("dock manager").graph,
        snapshot.root,
        dock_bounds,
        bounds,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        hint_font_size_inner,
        hint_font_size_outer,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        dragged_tab_for_drop,
        diagnostics_enabled.then_some(&mut candidates),
    );
    let mut auto_scrolled = false;
    if let Some(DockDropTarget::Dock(target)) = hover.as_mut() {
        let target_tabs = target.tabs;
        let tabs_len =
            app.global::<DockManager>()
                .and_then(|dock| match dock.graph.node(target_tabs) {
                    Some(fret_core::DockNode::Tabs { tabs, .. }) => Some(tabs.len()),
                    _ => None,
                });
        let tabs_rect = snapshot.layout_all.get(&target_tabs).copied();
        let frame_id = app.frame_id();
        let should_scroll = tabs_len.is_some()
            && tabs_rect.is_some()
            && app.with_global_mut(
                DeclarativeDockInteractionService::default,
                |service, _app| service.should_auto_scroll_tab_drag(window, target_tabs, frame_id),
            );
        if let (true, Some(tabs_len), Some(tabs_rect)) = (should_scroll, tabs_len, tabs_rect) {
            let (tab_bar, _content) = super::layout::split_tab_bar(tabs_rect);
            auto_scrolled = declarative_apply_tab_bar_drag_auto_scroll(
                theme.clone(),
                target,
                tab_bar,
                tabs_len,
                font_size,
                position,
                &tab_widths,
                &mut tab_scroll,
                dragged_tab_for_drop,
            );
        }
    }

    let (changed, graph_stats, graph_signature, diagnostics) =
        app.with_global_mut(DockManager::default, |dock, _app| {
            let changed = dock.hover != hover;
            dock.hover = hover;
            let graph_stats = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_stats_for_window(&dock.graph, window));
            let graph_signature = diagnostics_enabled
                .then(|| super::diagnostics::dock_graph_signature_for_window(&dock.graph, window));
            let diagnostics = diagnostics_enabled.then(|| {
                compute_dock_drop_resolve_diagnostics(
                    pointer_id,
                    position,
                    bounds,
                    dock_bounds,
                    source,
                    &dock.graph,
                    window,
                    dock.hover.as_ref(),
                    candidates,
                )
            });
            (changed, graph_stats, graph_signature, diagnostics)
        });
    if auto_scrolled {
        declarative_sync_tab_scroll_for_window(
            app,
            window,
            &tab_scroll,
            snapshot.layout_all.keys().copied(),
        );
    }
    let frame_id = app.frame_id();
    if let Some(dock_drop_resolve) = diagnostics {
        app.with_global_mut_untracked(
            fret_runtime::WindowInteractionDiagnosticsStore::default,
            |svc, _app| {
                svc.record_docking(
                    window,
                    frame_id,
                    fret_runtime::DockingInteractionDiagnostics {
                        dock_drop_resolve: Some(dock_drop_resolve),
                        dock_graph_stats: graph_stats,
                        dock_graph_signature: graph_signature,
                        ..Default::default()
                    },
                );
            },
        );
    }
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) && changed {
        let target = app
            .global::<DockManager>()
            .and_then(|dock| dock_drop_target_diagnostics(dock.hover.as_ref()));
        tracing::info!(
            window = ?window,
            invert_docking = !dock_previews_enabled,
            source = ?source,
            target = ?target,
            "declarative dock drag hover changed"
        );
    }
    (Vec::new(), changed || auto_scrolled, false)
}

fn declarative_panel_drag_allowed<H: UiHost>(
    app: &H,
    window: AppWindowId,
    panel: &PanelKey,
) -> bool {
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    let info = app
        .global::<DockManager>()
        .and_then(|dock| dock.panel(panel));
    policy
        .as_deref()
        .is_none_or(|policy| policy.allow_panel_drag(window, panel, info))
}

fn declarative_tabs_group_drag_allowed<H: UiHost>(
    app: &H,
    window: AppWindowId,
    tabs: fret_core::DockNodeId,
) -> bool {
    let policy = app
        .global::<DockingPolicyService>()
        .and_then(|service| service.policy());
    policy
        .as_deref()
        .is_none_or(|policy| policy.allow_tabs_group_drag(window, tabs))
}

fn begin_declarative_panel_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    pending: DeclarativePendingDockDrag,
    position: fret_core::Point,
    modifiers: fret_core::Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_PANEL,
        window,
        pending.start,
        position,
        DockPanelDragPayload {
            panel: pending.panel,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}

fn begin_declarative_tabs_group_drag<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    pointer_id: fret_core::PointerId,
    pending: DeclarativePendingDockTabsDrag,
    position: fret_core::Point,
    modifiers: fret_core::Modifiers,
) {
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let wants_dock_previews = settings.drag_inversion.wants_dock_previews(modifiers);
    let (tabs, active) = app
        .global::<DockManager>()
        .and_then(|dock| match dock.graph.node(pending.tabs) {
            Some(fret_core::DockNode::Tabs { tabs, active }) => Some((tabs.clone(), *active)),
            _ => None,
        })
        .unwrap_or_else(|| (Vec::new(), 0));
    let grab_offset = pending.grab_offset;
    begin_cross_window_dock_drag(
        app,
        pointer_id,
        fret_runtime::DRAG_KIND_DOCK_TABS,
        window,
        pending.start,
        position,
        DockTabsDragPayload {
            source_tabs: pending.tabs,
            tabs,
            active,
            grab_offset,
            tear_off_requested: false,
            tear_off_requested_at_tick: None,
            tear_off_oob_start_frame: None,
            dock_previews_enabled: wants_dock_previews,
        },
        None,
        grab_offset,
    );
}

fn apply_declarative_tab_interaction_paint_state(
    frame: &DockSpaceElementFrame,
    hover: DeclarativeTabHover,
    menu: Option<TabOverflowMenuState>,
    tab_chrome_inputs: &mut [TabChromePaintInput],
    tab_detail_inputs: &mut [TabDetailPaintInput],
) {
    for input in tab_chrome_inputs.iter_mut() {
        input.hovered_tab = None;
    }
    for input in tab_detail_inputs.iter_mut() {
        input.hovered_tab = None;
        input.hovered_tab_close = false;
        input.hovered_tab_overflow_button = false;
        input.tab_overflow_menu = None;
    }

    if let Some((tabs, index)) = hover.tab
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_chrome_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
        }
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab = Some(index);
            input.hovered_tab_close = hover.tab_close;
        }
    }

    if let Some(tabs) = hover.overflow_button
        && let Some(&rect) = frame.layout_all.get(&tabs)
    {
        for input in tab_detail_inputs
            .iter_mut()
            .filter(|input| input.rect == rect)
        {
            input.hovered_tab_overflow_button = true;
        }
    }

    if let Some(menu) = menu
        && let Some(&rect) = frame.layout_all.get(&menu.tabs)
        && let Some(input) = tab_detail_inputs
            .iter_mut()
            .find(|input| input.rect == rect)
    {
        input.tab_overflow_menu = Some(menu);
    }
}

fn drop_hints_from_hover(hover: Option<&super::types::DockDropTarget>) -> Option<DockDropHints> {
    let Some(super::types::DockDropTarget::Dock(target)) = hover else {
        return None;
    };
    Some(DockDropHints {
        root: target.root,
        leaf_tabs: target.leaf_tabs,
    })
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
                true,
            );
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let pressed_floating_close =
                declarative_pressed_floating_close_for_window(cx.app(), window);
            let floating_chrome_inputs =
                floating_chrome_paint_inputs(&snapshot, pressed_floating_close, floating_hover);
            let dock_drag_ghost = dock_drag_ghost_for_window(cx.app(), window);
            let (
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                complex_drop_overlay_inputs,
                split_handle_inputs,
                viewport_surface_inputs,
            ) = cx
                .app()
                .global::<DockManager>()
                .map(|dock| {
                    (
                        dock.hover.clone(),
                        tab_chrome_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                        ),
                        tab_detail_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                            tab_hover.tab_close,
                            tab_hover.overflow_button,
                            None,
                            tab_overflow_menu.clone(),
                        ),
                        complex_drop_overlay_paint_inputs(
                            theme.clone(),
                            dock.hover.clone(),
                            window,
                            &dock.graph,
                            &snapshot.layout_all,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &tab_scroll,
                            &tab_widths,
                        ),
                        split_handle_paint_inputs(&dock.graph, &snapshot.layout_all),
                        viewport_surface_paint_inputs(dock, window, &snapshot.layout_all),
                    )
                })
                .unwrap_or_default();
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

            cx.set_output(DockSpaceElementFrame::from_snapshot(
                &snapshot,
                panel_last_sizes,
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                tab_widths,
                tab_scroll,
                complex_drop_overlay_inputs,
                floating_chrome_inputs,
                dock_drag_ghost,
                split_handle_inputs,
                viewport_surface_inputs,
            ));
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
            declarative_sync_tab_scroll_for_window(
                cx.app(),
                window,
                &tab_scroll,
                snapshot.layout_all.keys().copied(),
            );
            let tab_overflow_menu = declarative_tab_overflow_menu_for_window(cx.app(), window);
            let tab_hover = declarative_tab_hover_for_window(cx.app(), window);
            let floating_hover = declarative_floating_hover_for_window(cx.app(), window);
            let pressed_floating_close =
                declarative_pressed_floating_close_for_window(cx.app(), window);
            let floating_chrome_inputs =
                floating_chrome_paint_inputs(&snapshot, pressed_floating_close, floating_hover);
            let dock_drag_ghost = dock_drag_ghost_for_window(cx.app(), window);
            let (
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                complex_drop_overlay_inputs,
                split_handle_inputs,
                viewport_surface_inputs,
            ) = cx
                .app()
                .global::<DockManager>()
                .map(|dock| {
                    (
                        dock.hover.clone(),
                        tab_chrome_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                        ),
                        tab_detail_paint_inputs(
                            &dock.graph,
                            &snapshot.layout_all,
                            &tab_widths,
                            &tab_scroll,
                            tab_hover.tab,
                            tab_hover.tab_close,
                            tab_hover.overflow_button,
                            None,
                            tab_overflow_menu.clone(),
                        ),
                        complex_drop_overlay_paint_inputs(
                            theme.clone(),
                            dock.hover.clone(),
                            window,
                            &dock.graph,
                            &snapshot.layout_all,
                            settings.split_handle_gap,
                            settings.split_handle_hit_thickness,
                            &tab_scroll,
                            &tab_widths,
                        ),
                        split_handle_paint_inputs(&dock.graph, &snapshot.layout_all),
                        viewport_surface_paint_inputs(dock, window, &snapshot.layout_all),
                    )
                })
                .unwrap_or_default();
            sync_declarative_viewport_layouts(cx.app(), window, &snapshot);

            let panel_last_sizes = snapshot
                .active_panel_bounds
                .iter()
                .map(|(panel, rect)| (panel.clone(), rect.size))
                .collect();
            cx.set_output(DockSpaceElementFrame::from_snapshot(
                &snapshot,
                panel_last_sizes,
                hover,
                tab_chrome_inputs,
                tab_detail_inputs,
                tab_widths,
                tab_scroll,
                complex_drop_overlay_inputs,
                floating_chrome_inputs,
                dock_drag_ghost,
                split_handle_inputs,
                viewport_surface_inputs,
            ));
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
                && cx.app().drag(e.pointer_id).is_some_and(|drag| {
                    drag.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                        || drag.kind == fret_runtime::DRAG_KIND_DOCK_TABS
                })
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
