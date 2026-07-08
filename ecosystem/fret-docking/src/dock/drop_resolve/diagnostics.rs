// This file is part of the docking UI implementation.
//
// It owns drop target and preview diagnostics projection for resolved dock drops.

use super::super::prelude_core::*;
use super::super::types::{DockDropPolicyDecision, DockDropTarget, HoverTarget};
use super::transaction::{DockDropCommandKind, ResolvedDockDropTransaction};

pub(in crate::dock) fn dock_drop_target_diagnostics(
    target: Option<&DockDropTarget>,
) -> Option<fret_runtime::DockDropTargetDiagnostics> {
    match target {
        Some(DockDropTarget::Dock(t)) => Some(hover_target_diagnostics(*t)),
        _ => None,
    }
}

fn hover_target_diagnostics(target: HoverTarget) -> fret_runtime::DockDropTargetDiagnostics {
    fret_runtime::DockDropTargetDiagnostics {
        layout_root: target.root,
        tabs: target.tabs,
        zone: target.zone,
        insert_index: target.insert_index,
        outer: target.outer,
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn compute_dock_drop_resolve_diagnostics(
    pointer_id: fret_core::PointerId,
    position: Point,
    window_bounds: Rect,
    dock_bounds: Rect,
    graph: &DockGraph,
    window: fret_core::AppWindowId,
    transaction: &ResolvedDockDropTransaction,
    candidates: Vec<fret_runtime::DockDropCandidateRectDiagnostics>,
) -> fret_runtime::DockDropResolveDiagnostics {
    let preview = match transaction.target.target_ref() {
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
        source: transaction.target.source,
        resolved: dock_drop_target_diagnostics(transaction.target.target_ref()),
        denied: transaction.denied_target().map(hover_target_diagnostics),
        preview,
        policy: policy_diagnostics(transaction.policy()),
        command: command_diagnostics(transaction.command),
        commit_capable: transaction.commit_capable(),
        clears_hover: transaction.clears_hover(),
        invalidates_layout: transaction.invalidates_layout(),
        candidates,
    }
}

fn policy_diagnostics(
    policy: &DockDropPolicyDecision,
) -> fret_runtime::DockDropPolicyDecisionDiagnostics {
    match policy {
        DockDropPolicyDecision::NotApplicable => {
            fret_runtime::DockDropPolicyDecisionDiagnostics::NotApplicable
        }
        DockDropPolicyDecision::Allowed => fret_runtime::DockDropPolicyDecisionDiagnostics::Allowed,
        DockDropPolicyDecision::Denied { .. } => {
            fret_runtime::DockDropPolicyDecisionDiagnostics::DeniedDockingPolicy
        }
    }
}

fn command_diagnostics(
    command: DockDropCommandKind,
) -> fret_runtime::DockDropCommandKindDiagnostics {
    match command {
        DockDropCommandKind::None => fret_runtime::DockDropCommandKindDiagnostics::None,
        DockDropCommandKind::MovePanel => fret_runtime::DockDropCommandKindDiagnostics::MovePanel,
        DockDropCommandKind::MovePanelToEmptyDockSpace => {
            fret_runtime::DockDropCommandKindDiagnostics::MovePanelToEmptyDockSpace
        }
        DockDropCommandKind::MoveTabs => fret_runtime::DockDropCommandKindDiagnostics::MoveTabs,
        DockDropCommandKind::MoveTabsToEmptyDockSpace => {
            fret_runtime::DockDropCommandKindDiagnostics::MoveTabsToEmptyDockSpace
        }
        DockDropCommandKind::FloatPanelInWindow => {
            fret_runtime::DockDropCommandKindDiagnostics::FloatPanelInWindow
        }
        DockDropCommandKind::FloatTabsInWindow => {
            fret_runtime::DockDropCommandKindDiagnostics::FloatTabsInWindow
        }
        DockDropCommandKind::RequestFloatPanelToNewWindow => {
            fret_runtime::DockDropCommandKindDiagnostics::RequestFloatPanelToNewWindow
        }
        DockDropCommandKind::RequestFloatTabsToNewWindow => {
            fret_runtime::DockDropCommandKindDiagnostics::RequestFloatTabsToNewWindow
        }
    }
}
