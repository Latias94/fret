use fret_core::dock::DockGraph;
use fret_core::{AppWindowId, DockNode, DockNodeId, PanelKey};

use super::manager::DockManager;
use super::types::{DockDragGhostSnapshot, DockPanelDragPayload, DockTabsDragPayload};

pub(super) fn diagnostics_env_enabled() -> bool {
    std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty())
        || std::env::var_os("FRET_DIAG_DIR").is_some_and(|v| !v.is_empty())
}

pub(super) fn should_publish_docking_diagnostics<H: fret_ui::UiHost>(
    app: &H,
    diag_env_enabled: bool,
) -> bool {
    diag_env_enabled
        || app
            .global::<fret_runtime::WindowInteractionDiagnosticsStore>()
            .is_some()
}

fn diag_scale_factor_x1000(scale_factor: f32) -> u32 {
    if !scale_factor.is_finite() {
        return 0;
    }
    let v = (scale_factor * 1000.0).round();
    if v <= 0.0 {
        return 0;
    }
    if v >= u32::MAX as f32 {
        return u32::MAX;
    }
    v as u32
}

pub(super) fn dock_drag_payload_ghost_visible(
    drag: &fret_runtime::DragSession,
    window: fret_core::AppWindowId,
) -> bool {
    if !drag.dragging || drag.current_window != window || drag.moving_window.is_some() {
        return false;
    }
    if drag.payload::<DockPanelDragPayload>().is_some() {
        return true;
    }
    drag.payload::<DockTabsDragPayload>()
        .is_some_and(|payload| {
            payload
                .tabs
                .get(payload.active)
                .or_else(|| payload.tabs.first())
                .is_some()
        })
}

pub(super) fn dock_drag_ghost_snapshot_for_window(
    drag: &fret_runtime::DragSession,
    window: fret_core::AppWindowId,
) -> Option<DockDragGhostSnapshot> {
    if !dock_drag_payload_ghost_visible(drag, window) {
        return None;
    }
    if let Some(payload) = drag.payload::<DockPanelDragPayload>() {
        return Some(DockDragGhostSnapshot {
            panel: payload.panel.clone(),
            position: drag.position,
            grab_offset: payload.grab_offset,
        });
    }

    let payload = drag.payload::<DockTabsDragPayload>()?;
    let panel = payload
        .tabs
        .get(payload.active)
        .or_else(|| payload.tabs.first())?
        .clone();
    Some(DockDragGhostSnapshot {
        panel,
        position: drag.position,
        grab_offset: payload.grab_offset,
    })
}

fn dock_drag_diagnostics_for_window<H: fret_ui::UiHost>(
    app: &H,
    window: fret_core::AppWindowId,
) -> Option<fret_runtime::DockDragDiagnostics> {
    let pointer_id = app.find_drag_pointer_id(|d| {
        (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
            || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
            && (d.source_window == window || d.current_window == window)
    })?;
    let drag = app.drag(pointer_id)?;
    let window_metrics = app.global::<fret_core::WindowMetricsService>();
    let current_window_scale_factor_x1000 = window_metrics
        .and_then(|svc| svc.scale_factor(drag.current_window))
        .map(diag_scale_factor_x1000);
    let moving_window_scale_factor_x1000 = drag.moving_window.and_then(|w| {
        window_metrics
            .and_then(|svc| svc.scale_factor(w))
            .map(diag_scale_factor_x1000)
    });

    Some(fret_runtime::DockDragDiagnostics {
        pointer_id,
        source_window: drag.source_window,
        current_window: drag.current_window,
        position: drag.position,
        start_position: drag.start_position,
        cursor_grab_offset: drag.cursor_grab_offset,
        follow_window: drag.follow_window,
        cursor_screen_pos_raw_physical_px: drag.diag_cursor_screen_pos_raw_physical_px,
        cursor_screen_pos_used_physical_px: drag.diag_cursor_screen_pos_used_physical_px,
        cursor_screen_pos_was_clamped: drag.diag_cursor_screen_pos_was_clamped,
        cursor_override_active: drag.diag_cursor_override_active,
        current_window_outer_pos_physical_px: drag.diag_current_window_outer_pos_physical_px,
        current_window_decoration_offset_physical_px: drag
            .diag_current_window_decoration_offset_physical_px,
        current_window_client_origin_screen_physical_px: drag
            .diag_current_window_client_origin_screen_physical_px,
        current_window_client_origin_source_platform: drag
            .diag_current_window_client_origin_source_platform,
        current_window_scale_factor_x1000_from_runner: drag.diag_current_window_scale_factor_x1000,
        current_window_local_pos_from_screen_logical_px: drag
            .diag_current_window_local_pos_from_screen_logical_px,
        current_window_scale_factor_x1000,
        kind: drag.kind,
        dragging: drag.dragging,
        cross_window_hover: drag.cross_window_hover,
        payload_ghost_visible: dock_drag_payload_ghost_visible(drag, window),
        transparent_payload_applied: drag.transparent_payload_applied,
        transparent_payload_hit_test_passthrough_applied: drag
            .transparent_payload_hit_test_passthrough_applied,
        window_under_cursor_source: drag.window_under_cursor_source,
        moving_window: drag.moving_window,
        moving_window_outer_pos_physical_px: drag.diag_moving_window_outer_pos_physical_px,
        moving_window_decoration_offset_physical_px: drag
            .diag_moving_window_decoration_offset_physical_px,
        moving_window_client_origin_screen_physical_px: drag
            .diag_moving_window_client_origin_screen_physical_px,
        moving_window_client_origin_source_platform: drag
            .diag_moving_window_client_origin_source_platform,
        moving_window_scale_factor_x1000_from_runner: drag.diag_moving_window_scale_factor_x1000,
        moving_window_local_pos_from_screen_logical_px: drag
            .diag_moving_window_local_pos_from_screen_logical_px,
        moving_window_scale_factor_x1000,
        window_under_moving_window: drag.window_under_moving_window,
        window_under_moving_window_source: drag.window_under_moving_window_source,
    })
}

#[derive(Debug, Clone, Default)]
pub(super) struct DockingDiagnosticsExtras {
    pub floating_drag: Option<fret_runtime::DockFloatingDragDiagnostics>,
    pub dock_drop_resolve: Option<fret_runtime::DockDropResolveDiagnostics>,
    pub viewport_capture: Option<fret_runtime::ViewportCaptureDiagnostics>,
    pub tab_strip_active_visibility: Option<fret_runtime::DockTabStripActiveVisibilityDiagnostics>,
}

pub(super) fn publish_docking_diagnostics_snapshot<H: fret_ui::UiHost>(
    app: &mut H,
    window: fret_core::AppWindowId,
    frame_id: fret_runtime::FrameId,
    extras: DockingDiagnosticsExtras,
) {
    let dock_drag = dock_drag_diagnostics_for_window(app, window);
    let (dock_graph_stats, dock_graph_signature) = app
        .global::<DockManager>()
        .map(|dock| {
            (
                Some(dock_graph_stats_for_window(&dock.workspace.graph, window)),
                Some(dock_graph_signature_for_window(
                    &dock.workspace.graph,
                    window,
                )),
            )
        })
        .unwrap_or((None, None));

    app.with_global_mut_untracked(
        fret_runtime::WindowInteractionDiagnosticsStore::default,
        |svc, _app| {
            svc.record_docking(
                window,
                frame_id,
                fret_runtime::DockingInteractionDiagnostics {
                    dock_drag,
                    floating_drag: extras.floating_drag,
                    dock_drop_resolve: extras.dock_drop_resolve,
                    viewport_capture: extras.viewport_capture,
                    tab_strip_active_visibility: extras.tab_strip_active_visibility,
                    dock_graph_stats,
                    dock_graph_signature,
                },
            );
        },
    );
}

/// Lightweight dock-graph diagnostics helpers.
///
/// These functions are primarily intended for scripted diagnostics gates and debugging tools.
/// They are **not** a stable public contract for persisted layouts.
pub fn dock_graph_stats_for_window(
    graph: &DockGraph,
    window: AppWindowId,
) -> fret_runtime::DockGraphStatsDiagnostics {
    use std::collections::HashSet;

    let mut node_count: u32 = 0;
    let mut tabs_count: u32 = 0;
    let mut split_count: u32 = 0;
    let mut floating_count: u32 = 0;
    let mut max_depth: u32 = 0;
    let mut max_split_depth: u32 = 0;

    let mut canonical_ok = true;
    let mut has_nested_same_axis_splits = false;

    let mut visited: HashSet<DockNodeId> = HashSet::new();
    let mut stack: Vec<(DockNodeId, u32, u32)> = Vec::new();

    if let Some(root) = graph.window_root(window) {
        stack.push((root, 1, 0));
    }
    for f in graph.floating_windows(window) {
        stack.push((f.floating, 1, 0));
    }

    while let Some((node, depth, split_depth)) = stack.pop() {
        if !visited.insert(node) {
            continue;
        }
        node_count = node_count.saturating_add(1);
        max_depth = max_depth.max(depth);
        max_split_depth = max_split_depth.max(split_depth);

        let Some(n) = graph.node(node) else {
            canonical_ok = false;
            continue;
        };

        match n {
            DockNode::Tabs { tabs, .. } => {
                tabs_count = tabs_count.saturating_add(1);
                if tabs.is_empty() {
                    canonical_ok = false;
                }
            }
            DockNode::Floating { child } => {
                floating_count = floating_count.saturating_add(1);
                stack.push((*child, depth.saturating_add(1), split_depth));
            }
            DockNode::Split {
                axis,
                children,
                fractions,
            } => {
                split_count = split_count.saturating_add(1);

                if children.len() < 2 || children.len() != fractions.len() {
                    canonical_ok = false;
                }

                let mut sum: f32 = 0.0;
                for f in fractions {
                    if !f.is_finite() || *f < 0.0 {
                        canonical_ok = false;
                    }
                    sum += *f;
                }
                if !sum.is_finite() || (sum - 1.0).abs() > 1.0e-3 {
                    canonical_ok = false;
                }

                for &child in children {
                    if let Some(DockNode::Split {
                        axis: child_axis, ..
                    }) = graph.node(child)
                        && child_axis == axis
                    {
                        has_nested_same_axis_splits = true;
                        canonical_ok = false;
                    }
                    stack.push((
                        child,
                        depth.saturating_add(1),
                        split_depth.saturating_add(1),
                    ));
                }
            }
        }
    }

    fret_runtime::DockGraphStatsDiagnostics {
        node_count,
        tabs_count,
        split_count,
        floating_count,
        max_depth,
        max_split_depth,
        canonical_ok,
        has_nested_same_axis_splits,
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn dock_graph_signature_for_window(
    graph: &DockGraph,
    window: AppWindowId,
) -> fret_runtime::DockGraphSignatureDiagnostics {
    use std::collections::HashSet;

    fn panel_key_sig(p: &PanelKey) -> String {
        match &p.instance {
            Some(instance) if !instance.is_empty() => format!("{}#{}", p.kind.0, instance),
            _ => p.kind.0.clone(),
        }
    }

    fn node_sig(graph: &DockGraph, node: DockNodeId, visited: &mut HashSet<DockNodeId>) -> String {
        if !visited.insert(node) {
            return "cycle".to_string();
        }

        let Some(n) = graph.node(node) else {
            return "missing".to_string();
        };

        match n {
            DockNode::Tabs { tabs, active } => {
                let body = tabs.iter().map(panel_key_sig).collect::<Vec<_>>().join(",");
                if tabs.len() > 1 {
                    format!("tabs(a={active}:[{body}])")
                } else {
                    format!("tabs([{body}])")
                }
            }
            DockNode::Floating { child } => {
                let child_sig = node_sig(graph, *child, visited);
                format!("floating({child_sig})")
            }
            DockNode::Split { axis, children, .. } => {
                let axis = match axis {
                    fret_core::Axis::Horizontal => "h",
                    fret_core::Axis::Vertical => "v",
                };
                let child_sigs = children
                    .iter()
                    .map(|c| node_sig(graph, *c, visited))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("split({axis},[{child_sigs}])")
            }
        }
    }

    let root_sig = graph
        .window_root(window)
        .map(|root| node_sig(graph, root, &mut HashSet::new()))
        .unwrap_or_else(|| "none".to_string());

    let mut floating_sigs: Vec<String> = graph
        .floating_windows(window)
        .iter()
        .map(|f| node_sig(graph, f.floating, &mut HashSet::new()))
        .collect();
    floating_sigs.sort();

    let signature = format!(
        "dock(root={root_sig};floatings=[{}])",
        floating_sigs.join(",")
    );
    let fingerprint64 = fnv1a64(signature.as_bytes());

    fret_runtime::DockGraphSignatureDiagnostics {
        signature,
        fingerprint64,
    }
}
