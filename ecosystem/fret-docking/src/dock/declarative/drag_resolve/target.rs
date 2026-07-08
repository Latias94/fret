use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, DockNodeId, Point, Px, Rect};
use fret_ui::{ThemeSnapshot, UiHost};

use super::super::super::drop_resolve::resolve_dock_drop_target;
use super::super::super::host_frame::DockSpaceLayoutSnapshot;
use super::super::super::layout::dock_space_regions;
use super::super::super::manager::DockManager;
use super::super::super::services::DockingPolicyService;
use super::super::super::types::{DockDropTarget, DockPanelDragPayload};
use super::super::geometry::declarative_layout_snapshot_for_bounds;
use super::super::tab_metrics::{
    declarative_tab_scroll_for_frame, declarative_tab_widths_for_layout,
};

// This file owns declarative docking drop-target resolution input projection.

pub(super) struct DeclarativeDragTargetResolution {
    pub(super) snapshot: DockSpaceLayoutSnapshot,
    pub(super) dock_bounds: Rect,
    pub(super) font_size: Px,
    pub(super) tab_widths: HashMap<DockNodeId, Arc<[Px]>>,
    pub(super) tab_scroll: HashMap<DockNodeId, Px>,
    pub(super) target: Option<DockDropTarget>,
    pub(super) source: fret_runtime::DockDropResolveSource,
    pub(super) candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
}

pub(super) fn declarative_dragged_tab_for_drop<H: UiHost>(
    app: &H,
    drag: &fret_runtime::DragSession,
) -> Option<(DockNodeId, usize)> {
    let payload = drag.payload::<DockPanelDragPayload>()?;
    app.global::<DockManager>()?
        .workspace
        .graph
        .find_panel_in_window(drag.source_window, &payload.panel)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_declarative_drag_target<H: UiHost>(
    app: &H,
    window: AppWindowId,
    bounds: Rect,
    theme: ThemeSnapshot,
    position: Point,
    dock_previews_enabled: bool,
    prev_hover: Option<DockDropTarget>,
    dragged_tab_for_drop: Option<(DockNodeId, usize)>,
    diagnostics_enabled: bool,
) -> Option<DeclarativeDragTargetResolution> {
    let snapshot = declarative_layout_snapshot_for_bounds(app, window, bounds)?;
    let (_chrome, dock_bounds) = dock_space_regions(bounds);
    let settings = app
        .global::<fret_runtime::DockingInteractionSettings>()
        .copied()
        .unwrap_or_default();
    let font_size = theme.metric_token("font.size");
    let hint_font_size_inner = Px((font_size.0 * settings.dock_hint_scale_inner.max(0.0)).max(0.0));
    let hint_font_size_outer = Px((font_size.0 * settings.dock_hint_scale_outer.max(0.0)).max(0.0));
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
    let mut candidates = Vec::<fret_runtime::DockDropCandidateRectDiagnostics>::new();
    let graph = &app
        .global::<DockManager>()
        .expect("dock manager")
        .workspace
        .graph;
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
        theme,
        hint_font_size_inner,
        hint_font_size_outer,
        snapshot.split_handle_gap,
        snapshot.split_handle_hit_thickness,
        position,
        dragged_tab_for_drop,
        diagnostics_enabled.then_some(&mut candidates),
    );

    Some(DeclarativeDragTargetResolution {
        snapshot,
        dock_bounds,
        font_size,
        tab_widths,
        tab_scroll,
        target,
        source,
        candidates,
    })
}
