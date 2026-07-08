use fret_core::{AppWindowId, Point, PointerId, Rect};
use fret_ui::UiHost;

use super::super::super::diagnostics::{
    dock_graph_signature_for_window, dock_graph_stats_for_window,
};
use super::super::super::drop_resolve::ResolvedDockDropTransaction;
use super::super::super::drop_resolve::compute_dock_drop_resolve_diagnostics;
use super::super::super::manager::DockManager;

// This file owns declarative docking drag resolve diagnostics capture and publication.

pub(super) struct DragResolveDiagnosticsCapture {
    graph_stats: Option<fret_runtime::DockGraphStatsDiagnostics>,
    graph_signature: Option<fret_runtime::DockGraphSignatureDiagnostics>,
    dock_drop_resolve: Option<fret_runtime::DockDropResolveDiagnostics>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn capture_drag_drop_diagnostics<H: UiHost>(
    app: &mut H,
    diagnostics_enabled: bool,
    pointer_id: PointerId,
    position: Point,
    bounds: Rect,
    dock_bounds: Rect,
    window: AppWindowId,
    transaction: &ResolvedDockDropTransaction,
    candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
) -> DragResolveDiagnosticsCapture {
    app.with_global_mut(DockManager::default, |dock, _app| {
        dock.presentation.hover = None;
        let graph_stats =
            diagnostics_enabled.then(|| dock_graph_stats_for_window(&dock.workspace.graph, window));
        let graph_signature = diagnostics_enabled
            .then(|| dock_graph_signature_for_window(&dock.workspace.graph, window));
        let dock_drop_resolve = diagnostics_enabled.then(|| {
            compute_dock_drop_resolve_diagnostics(
                pointer_id,
                position,
                bounds,
                dock_bounds,
                &dock.workspace.graph,
                window,
                transaction,
                candidates,
            )
        });
        DragResolveDiagnosticsCapture {
            graph_stats,
            graph_signature,
            dock_drop_resolve,
        }
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_hover_and_capture_diagnostics<H: UiHost>(
    app: &mut H,
    diagnostics_enabled: bool,
    transaction: &ResolvedDockDropTransaction,
    pointer_id: PointerId,
    position: Point,
    bounds: Rect,
    dock_bounds: Rect,
    window: AppWindowId,
    candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
) -> (bool, DragResolveDiagnosticsCapture) {
    app.with_global_mut(DockManager::default, |dock, _app| {
        let hover = transaction.target.target.clone();
        let changed = dock.presentation.hover != hover;
        dock.presentation.hover = hover;
        let graph_stats =
            diagnostics_enabled.then(|| dock_graph_stats_for_window(&dock.workspace.graph, window));
        let graph_signature = diagnostics_enabled
            .then(|| dock_graph_signature_for_window(&dock.workspace.graph, window));
        let dock_drop_resolve = diagnostics_enabled.then(|| {
            compute_dock_drop_resolve_diagnostics(
                pointer_id,
                position,
                bounds,
                dock_bounds,
                &dock.workspace.graph,
                window,
                transaction,
                candidates,
            )
        });
        (
            changed,
            DragResolveDiagnosticsCapture {
                graph_stats,
                graph_signature,
                dock_drop_resolve,
            },
        )
    })
}

pub(super) fn record_drag_resolve_diagnostics<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    capture: DragResolveDiagnosticsCapture,
) {
    let Some(dock_drop_resolve) = capture.dock_drop_resolve else {
        return;
    };
    let frame_id = app.frame_id();
    app.with_global_mut_untracked(
        fret_runtime::WindowInteractionDiagnosticsStore::default,
        |svc, _app| {
            svc.record_docking(
                window,
                frame_id,
                fret_runtime::DockingInteractionDiagnostics {
                    dock_drop_resolve: Some(dock_drop_resolve),
                    dock_graph_stats: capture.graph_stats,
                    dock_graph_signature: capture.graph_signature,
                    ..Default::default()
                },
            );
        },
    );
}
