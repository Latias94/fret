use std::collections::HashMap;

use fret_core::{AppWindowId, PanelKey, Rect, Size};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::super::diagnostics::{diagnostics_env_enabled, should_publish_docking_diagnostics};
use super::super::drop_resolve::{
    DockPanelDropDrag, DockTabsDropDrag, apply_dock_drop_intent, dock_drop_intent_debug_kind,
    dock_drop_target_diagnostics, resolve_dock_drop_intent_panel, resolve_dock_drop_intent_tabs,
    resolve_dock_drop_target,
};
use super::super::layout::{dock_space_regions, split_tab_bar};
use super::super::manager::DockManager;
use super::super::services::DockingPolicyService;
use super::super::types::{DockDropTarget, DockPanelDragPayload, DockTabsDragPayload};
use super::geometry::declarative_layout_snapshot_for_bounds;
use super::interaction::DeclarativeDockInteractionService;
use super::tab_metrics::{
    declarative_apply_tab_bar_drag_auto_scroll, declarative_sync_tab_scroll_for_window,
    declarative_tab_scroll_for_frame, declarative_tab_widths_for_layout,
};
use super::tear_off::{
    declarative_allow_tear_off_for_panel, declarative_default_floating_rect_for_panel,
    declarative_resolve_tear_off_hover,
};

mod begin_drag;
mod diagnostics;

pub(super) use begin_drag::{begin_declarative_panel_drag, begin_declarative_tabs_group_drag};
use diagnostics::{
    capture_drag_drop_diagnostics, record_drag_resolve_diagnostics,
    update_hover_and_capture_diagnostics,
};

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
pub(super) fn declarative_resolve_internal_drag_drop<H: UiHost>(
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
        super::super::types::DockDropIntent::None
    };
    apply_dock_drop_intent(intent.clone(), &mut effects, &mut invalidate_layout);

    let diagnostics = capture_drag_drop_diagnostics(
        app,
        diagnostics_enabled,
        pointer_id,
        position,
        bounds,
        dock_bounds,
        source,
        window,
        target.as_ref(),
        candidates,
    );
    record_drag_resolve_diagnostics(app, window, diagnostics);
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

#[allow(clippy::too_many_arguments)]
pub(super) fn declarative_resolve_internal_drag_hover<H: UiHost>(
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
            let (tab_bar, _content) = split_tab_bar(tabs_rect);
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

    let (changed, diagnostics) = update_hover_and_capture_diagnostics(
        app,
        diagnostics_enabled,
        hover,
        pointer_id,
        position,
        bounds,
        dock_bounds,
        source,
        window,
        candidates,
    );
    if auto_scrolled {
        declarative_sync_tab_scroll_for_window(
            app,
            window,
            &tab_scroll,
            snapshot.layout_all.keys().copied(),
        );
    }
    record_drag_resolve_diagnostics(app, window, diagnostics);
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

pub(super) fn declarative_panel_drag_allowed<H: UiHost>(
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

pub(super) fn declarative_tabs_group_drag_allowed<H: UiHost>(
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
