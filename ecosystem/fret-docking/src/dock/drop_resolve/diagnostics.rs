// This file is part of the docking UI implementation.
//
// It owns drop target and preview diagnostics projection for resolved dock drops.

use super::super::prelude_core::*;
use super::super::types::DockDropTarget;

pub(in crate::dock) fn dock_drop_target_diagnostics(
    target: Option<&DockDropTarget>,
) -> Option<fret_runtime::DockDropTargetDiagnostics> {
    match target {
        Some(DockDropTarget::Dock(t)) => Some(fret_runtime::DockDropTargetDiagnostics {
            layout_root: t.root,
            tabs: t.tabs,
            zone: t.zone,
            insert_index: t.insert_index,
            outer: t.outer,
        }),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn compute_dock_drop_resolve_diagnostics(
    pointer_id: fret_core::PointerId,
    position: Point,
    window_bounds: Rect,
    dock_bounds: Rect,
    source: fret_runtime::DockDropResolveSource,
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    target: Option<&DockDropTarget>,
    candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
) -> fret_runtime::DockDropResolveDiagnostics {
    let preview = match target {
        Some(DockDropTarget::Dock(t)) if t.zone != DropZone::Center => {
            let kind = match graph.edge_dock_decision(window, t.tabs, t.zone) {
                Some(fret_core::EdgeDockDecision::InsertIntoSplit {
                    split,
                    insert_index,
                    ..
                }) => {
                    let axis = match graph.node(split) {
                        Some(DockNode::Split { axis, .. }) => *axis,
                        _ => match t.zone {
                            DropZone::Left | DropZone::Right => fret_core::Axis::Horizontal,
                            DropZone::Top | DropZone::Bottom => fret_core::Axis::Vertical,
                            DropZone::Center => fret_core::Axis::Horizontal,
                        },
                    };
                    fret_runtime::DockDropPreviewKindDiagnostics::InsertIntoSplit {
                        axis,
                        split,
                        insert_index,
                    }
                }
                _ => fret_runtime::DockDropPreviewKindDiagnostics::WrapBinary,
            };
            Some(fret_runtime::DockDropPreviewDiagnostics { kind })
        }
        _ => None,
    };

    fret_runtime::DockDropResolveDiagnostics {
        pointer_id,
        position,
        window_bounds,
        dock_bounds,
        source,
        resolved: dock_drop_target_diagnostics(target),
        preview,
        candidates,
    }
}
