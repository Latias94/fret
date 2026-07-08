use fret_core::{AppWindowId, PanelKey, Rect};
use fret_runtime::Effect;
use fret_ui::UiHost;

use super::super::diagnostics::{diagnostics_env_enabled, should_publish_docking_diagnostics};
use super::super::drop_resolve::{
    apply_resolved_dock_drop_transaction, dock_drop_target_diagnostics,
    dock_drop_transaction_debug_kind, resolve_dock_drop_transaction,
};
use super::super::manager::DockManager;
use super::super::services::DockingPolicyService;
use super::super::types::{DockPanelDragPayload, DockTabsDragPayload};
use super::tear_off::declarative_resolve_tear_off_hover;

mod begin_drag;
mod diagnostics;
mod drop_intent;
mod hover_autoscroll;
mod target;

pub(super) use begin_drag::{begin_declarative_panel_drag, begin_declarative_tabs_group_drag};
use diagnostics::{
    capture_drag_drop_diagnostics, record_drag_resolve_diagnostics,
    update_hover_and_capture_diagnostics,
};
use drop_intent::resolve_declarative_drag_drop_intent;
use hover_autoscroll::apply_drag_hover_auto_scroll;
use target::{declarative_dragged_tab_for_drop, resolve_declarative_drag_target};

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
            dock.presentation.hover.take().is_some()
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
            dock.presentation.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    }

    let source_window = drag.source_window;
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();

    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let prev_hover = app
        .global::<DockManager>()
        .and_then(|dock| dock.presentation.hover.clone());
    let Some(target_resolution) = resolve_declarative_drag_target(
        app,
        window,
        bounds,
        theme,
        position,
        dock_previews_enabled,
        prev_hover,
        dragged_tab_for_drop,
        diagnostics_enabled,
    ) else {
        let hover_cleared = app.with_global_mut(DockManager::default, |dock, _app| {
            dock.presentation.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false, true);
    };

    let mut effects = Vec::new();
    let intent = resolve_declarative_drag_drop_intent(
        app,
        target_resolution.drop_target.target_ref(),
        panel_payload.as_ref(),
        tabs_payload.as_ref(),
        source_window,
        window,
        bounds,
        position,
        allow_tear_off,
        allow_multi_window_tear_off,
        &target_resolution.snapshot.paint_panel_bounds,
    );
    let transaction = resolve_dock_drop_transaction(target_resolution.drop_target.clone(), intent);
    let applied = apply_resolved_dock_drop_transaction(app, &transaction, &mut effects);
    let invalidate_layout = applied && transaction.invalidates_layout();

    let diagnostics = capture_drag_drop_diagnostics(
        app,
        diagnostics_enabled,
        pointer_id,
        position,
        bounds,
        target_resolution.dock_bounds,
        window,
        &transaction,
        target_resolution.candidates,
    );
    record_drag_resolve_diagnostics(app, window, diagnostics);
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) {
        let drop_target_diag = dock_drop_target_diagnostics(transaction.target.target_ref());
        tracing::info!(
            window = ?window,
            source_window = ?source_window,
            pointer_id = ?pointer_id,
            pos = ?position,
            invert_docking = !dock_previews_enabled,
            resolve_source = ?transaction.target.source,
            drop_target = ?drop_target_diag,
            intent_kind = dock_drop_transaction_debug_kind(&transaction),
            commit_capable = transaction.commit_capable(),
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
            dock.presentation.hover.take().is_some()
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
            dock.presentation.hover.take().is_some()
        });
        return (Vec::new(), hover_cleared, false);
    }
    let source_window = drag.source_window;
    let dragged_tab_for_drop = declarative_dragged_tab_for_drop(app, drag);
    let panel_payload = drag.payload::<DockPanelDragPayload>().cloned();
    let tabs_payload = drag.payload::<DockTabsDragPayload>().cloned();

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
            dock.presentation.hover.take().is_some()
        });
        return (Vec::new(), true, true);
    }
    let diagnostics_enabled = should_publish_docking_diagnostics(app, diagnostics_env_enabled());
    let Some(mut target_resolution) = resolve_declarative_drag_target(
        app,
        window,
        bounds,
        theme.clone(),
        position,
        dock_previews_enabled,
        None,
        dragged_tab_for_drop,
        diagnostics_enabled,
    ) else {
        return (Vec::new(), false, false);
    };
    let auto_scrolled = apply_drag_hover_auto_scroll(
        app,
        window,
        &mut target_resolution.drop_target.target,
        &target_resolution.snapshot.layout_all,
        theme.clone(),
        target_resolution.font_size,
        position,
        &target_resolution.tab_widths,
        &mut target_resolution.tab_scroll,
        dragged_tab_for_drop,
    );
    let intent = resolve_declarative_drag_drop_intent(
        app,
        target_resolution.drop_target.target_ref(),
        panel_payload.as_ref(),
        tabs_payload.as_ref(),
        source_window,
        window,
        bounds,
        position,
        allow_tear_off,
        allow_multi_window_tear_off,
        &target_resolution.snapshot.paint_panel_bounds,
    );
    let transaction = resolve_dock_drop_transaction(target_resolution.drop_target.clone(), intent);
    let resolve_source = target_resolution.drop_target.source;

    let (changed, diagnostics) = update_hover_and_capture_diagnostics(
        app,
        diagnostics_enabled,
        &transaction,
        pointer_id,
        position,
        bounds,
        target_resolution.dock_bounds,
        window,
        target_resolution.candidates,
    );
    record_drag_resolve_diagnostics(app, window, diagnostics);
    if std::env::var_os("FRET_DOCK_DRAG_DEBUG").is_some_and(|v| !v.is_empty()) && changed {
        let target = app
            .global::<DockManager>()
            .and_then(|dock| dock_drop_target_diagnostics(dock.presentation.hover.as_ref()));
        tracing::info!(
            window = ?window,
            invert_docking = !dock_previews_enabled,
            source = ?resolve_source,
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
