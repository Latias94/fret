// This file is part of the docking UI implementation.
//
// It owns the resolved drop transaction consumed by commit, preview, and diagnostics adapters.

use super::super::prelude_core::*;
use super::super::prelude_runtime::*;
use super::super::types::{
    DockDropIntent, DockDropPolicyDecision, DockDropTargetResolution, HoverTarget,
};
use fret_ui::UiHost;

#[derive(Debug, Clone, PartialEq)]
pub(in crate::dock) struct ResolvedDockDropTransaction {
    pub(in crate::dock) target: DockDropTargetResolution,
    pub(in crate::dock) intent: DockDropIntent,
    pub(in crate::dock) command: DockDropCommandKind,
    pub(in crate::dock) cleanup: DockDropCleanup,
}

impl ResolvedDockDropTransaction {
    pub(in crate::dock) fn commit_capable(&self) -> bool {
        self.command != DockDropCommandKind::None
    }

    pub(in crate::dock) fn invalidates_layout(&self) -> bool {
        self.cleanup.invalidates_layout
    }

    pub(in crate::dock) fn clears_hover(&self) -> bool {
        self.cleanup.clears_hover
    }

    pub(in crate::dock) fn policy(&self) -> &DockDropPolicyDecision {
        &self.target.policy
    }

    pub(in crate::dock) fn denied_target(&self) -> Option<HoverTarget> {
        match self.policy() {
            DockDropPolicyDecision::Denied { target } => Some(*target),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::dock) enum DockDropCommandKind {
    None,
    MovePanel,
    MovePanelToEmptyDockSpace,
    MoveTabs,
    MoveTabsToEmptyDockSpace,
    FloatPanelInWindow,
    FloatTabsInWindow,
    RequestFloatPanelToNewWindow,
    RequestFloatTabsToNewWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::dock) struct DockDropCleanup {
    pub(in crate::dock) clears_hover: bool,
    pub(in crate::dock) invalidates_layout: bool,
}

pub(in crate::dock) fn resolve_dock_drop_transaction(
    target: DockDropTargetResolution,
    intent: DockDropIntent,
) -> ResolvedDockDropTransaction {
    let command = dock_drop_command_kind(&intent);
    ResolvedDockDropTransaction {
        target,
        intent,
        command,
        cleanup: DockDropCleanup {
            clears_hover: true,
            invalidates_layout: command != DockDropCommandKind::None,
        },
    }
}

pub(in crate::dock) fn dock_drop_transaction_debug_kind(
    transaction: &ResolvedDockDropTransaction,
) -> &'static str {
    dock_drop_command_debug_kind(transaction.command)
}

fn dock_drop_command_kind(intent: &DockDropIntent) -> DockDropCommandKind {
    match intent {
        DockDropIntent::None => DockDropCommandKind::None,
        DockDropIntent::MovePanel { .. } => DockDropCommandKind::MovePanel,
        DockDropIntent::MovePanelToEmptyDockSpace { .. } => {
            DockDropCommandKind::MovePanelToEmptyDockSpace
        }
        DockDropIntent::MoveTabs { .. } => DockDropCommandKind::MoveTabs,
        DockDropIntent::MoveTabsToEmptyDockSpace { .. } => {
            DockDropCommandKind::MoveTabsToEmptyDockSpace
        }
        DockDropIntent::FloatPanelInWindow { .. } => DockDropCommandKind::FloatPanelInWindow,
        DockDropIntent::FloatTabsInWindow { .. } => DockDropCommandKind::FloatTabsInWindow,
        DockDropIntent::RequestFloatPanelToNewWindow { .. } => {
            DockDropCommandKind::RequestFloatPanelToNewWindow
        }
        DockDropIntent::RequestFloatTabsToNewWindow { .. } => {
            DockDropCommandKind::RequestFloatTabsToNewWindow
        }
    }
}

fn dock_drop_command_debug_kind(command: DockDropCommandKind) -> &'static str {
    match command {
        DockDropCommandKind::None => "none",
        DockDropCommandKind::MovePanel => "move_panel",
        DockDropCommandKind::MovePanelToEmptyDockSpace => "move_panel_to_empty_dock_space",
        DockDropCommandKind::MoveTabs => "move_tabs",
        DockDropCommandKind::MoveTabsToEmptyDockSpace => "move_tabs_to_empty_dock_space",
        DockDropCommandKind::FloatPanelInWindow => "float_panel_in_window",
        DockDropCommandKind::FloatTabsInWindow => "float_tabs_in_window",
        DockDropCommandKind::RequestFloatPanelToNewWindow => "request_float_panel_to_new_window",
        DockDropCommandKind::RequestFloatTabsToNewWindow => "request_float_tabs_to_new_window",
    }
}

pub(in crate::dock) fn apply_resolved_dock_drop_transaction<H: UiHost>(
    app: &mut H,
    transaction: &ResolvedDockDropTransaction,
    pending_effects: &mut Vec<Effect>,
) -> bool {
    match transaction.intent.clone() {
        DockDropIntent::None => false,
        DockDropIntent::MovePanel {
            source_window,
            panel,
            target_window,
            target_tabs,
            zone,
            insert_index,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MovePanel {
                source_window,
                panel,
                target_window,
                target_tabs,
                zone,
                insert_index,
            }));
            true
        }
        DockDropIntent::MovePanelToEmptyDockSpace {
            source_window,
            panel,
            target_window,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MovePanelToEmptyDockSpace {
                source_window,
                panel,
                target_window,
            }));
            true
        }
        DockDropIntent::MoveTabs {
            source_window,
            source_tabs,
            target_window,
            target_tabs,
            zone,
            insert_index,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MoveTabs {
                source_window,
                source_tabs,
                target_window,
                target_tabs,
                zone,
                insert_index,
            }));
            true
        }
        DockDropIntent::MoveTabsToEmptyDockSpace {
            source_window,
            source_tabs,
            target_window,
        } => {
            pending_effects.push(Effect::Dock(DockOp::MoveTabsToEmptyDockSpace {
                source_window,
                source_tabs,
                target_window,
            }));
            true
        }
        DockDropIntent::FloatPanelInWindow {
            source_window,
            panel,
            target_window,
            rect,
        } => {
            pending_effects.push(Effect::Dock(DockOp::FloatPanelInWindow {
                source_window,
                panel,
                target_window,
                rect,
            }));
            true
        }
        DockDropIntent::FloatTabsInWindow {
            source_window,
            source_tabs,
            target_window,
            rect,
        } => {
            pending_effects.push(Effect::Dock(DockOp::FloatTabsInWindow {
                source_window,
                source_tabs,
                target_window,
                rect,
            }));
            true
        }
        DockDropIntent::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } => crate::runtime::request_float_panel_to_new_window_with_host_effects(
            app,
            source_window,
            panel,
            anchor,
        ),
        DockDropIntent::RequestFloatTabsToNewWindow {
            source_window,
            source_tabs,
            panel,
            anchor,
        } => crate::runtime::request_float_tabs_to_new_window_with_host_effects(
            app,
            source_window,
            source_tabs,
            panel,
            anchor,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::AppWindowId;
    use slotmap::KeyData;

    fn window(raw: u64) -> AppWindowId {
        AppWindowId::from(KeyData::from_ffi(raw))
    }

    fn node(raw: u64) -> DockNodeId {
        DockNodeId::from(KeyData::from_ffi(raw))
    }

    fn panel(name: &str) -> PanelKey {
        PanelKey::new(name)
    }

    fn target_resolution(policy: DockDropPolicyDecision) -> DockDropTargetResolution {
        DockDropTargetResolution {
            target: None,
            source: fret_runtime::DockDropResolveSource::None,
            policy,
        }
    }

    fn diagnostics_rect() -> Rect {
        Rect::new(Point::default(), Size::new(Px(100.0), Px(100.0)))
    }

    fn graph_with_tabs(window: AppWindowId, panel: PanelKey) -> (DockGraph, DockNodeId) {
        let mut graph = DockGraph::new();
        let tabs = graph.insert_node(DockNode::Tabs {
            tabs: vec![panel],
            active: 0,
        });
        graph.set_window_root(window, tabs);
        (graph, tabs)
    }

    #[test]
    fn transaction_matrix_classifies_commit_and_cleanup_outcomes() {
        let source_window = window(1);
        let target_window = window(2);
        let target_tabs = node(1);
        let source_tabs = node(2);
        let floating_rect = Rect::new(Point::default(), Size::new(Px(10.0), Px(10.0)));
        let panel = panel("demo.panel");

        let cases = [
            (
                DockDropIntent::None,
                DockDropCommandKind::None,
                false,
                false,
            ),
            (
                DockDropIntent::MovePanel {
                    source_window,
                    panel: panel.clone(),
                    target_window,
                    target_tabs,
                    zone: DropZone::Center,
                    insert_index: Some(1),
                },
                DockDropCommandKind::MovePanel,
                true,
                true,
            ),
            (
                DockDropIntent::MovePanelToEmptyDockSpace {
                    source_window,
                    panel: panel.clone(),
                    target_window,
                },
                DockDropCommandKind::MovePanelToEmptyDockSpace,
                true,
                true,
            ),
            (
                DockDropIntent::MoveTabs {
                    source_window,
                    source_tabs,
                    target_window,
                    target_tabs,
                    zone: DropZone::Left,
                    insert_index: None,
                },
                DockDropCommandKind::MoveTabs,
                true,
                true,
            ),
            (
                DockDropIntent::MoveTabsToEmptyDockSpace {
                    source_window,
                    source_tabs,
                    target_window,
                },
                DockDropCommandKind::MoveTabsToEmptyDockSpace,
                true,
                true,
            ),
            (
                DockDropIntent::FloatPanelInWindow {
                    source_window,
                    panel: panel.clone(),
                    target_window,
                    rect: floating_rect,
                },
                DockDropCommandKind::FloatPanelInWindow,
                true,
                true,
            ),
            (
                DockDropIntent::FloatTabsInWindow {
                    source_window,
                    source_tabs,
                    target_window,
                    rect: floating_rect,
                },
                DockDropCommandKind::FloatTabsInWindow,
                true,
                true,
            ),
            (
                DockDropIntent::RequestFloatPanelToNewWindow {
                    source_window,
                    panel: panel.clone(),
                    anchor: None,
                },
                DockDropCommandKind::RequestFloatPanelToNewWindow,
                true,
                true,
            ),
            (
                DockDropIntent::RequestFloatTabsToNewWindow {
                    source_window,
                    source_tabs,
                    panel,
                    anchor: None,
                },
                DockDropCommandKind::RequestFloatTabsToNewWindow,
                true,
                true,
            ),
        ];

        for (intent, command, commit_capable, invalidates_layout) in cases {
            let tx = resolve_dock_drop_transaction(
                target_resolution(DockDropPolicyDecision::NotApplicable),
                intent,
            );
            assert_eq!(tx.command, command);
            assert_eq!(tx.commit_capable(), commit_capable);
            assert_eq!(tx.invalidates_layout(), invalidates_layout);
            assert!(tx.clears_hover());
        }
    }

    #[test]
    fn transaction_preserves_policy_denied_target_for_diagnostics() {
        let denied = HoverTarget {
            tabs: node(1),
            root: node(2),
            leaf_tabs: node(1),
            zone: DropZone::Right,
            insert_index: None,
            outer: true,
            explicit: true,
        };

        let tx = resolve_dock_drop_transaction(
            target_resolution(DockDropPolicyDecision::Denied { target: denied }),
            DockDropIntent::None,
        );

        assert_eq!(tx.command, DockDropCommandKind::None);
        assert!(!tx.commit_capable());
        assert_eq!(tx.denied_target(), Some(denied));
    }

    #[test]
    fn transaction_diagnostics_project_command_policy_cleanup_and_preview() {
        let window = window(1);
        let panel = panel("demo.panel");
        let (graph, tabs) = graph_with_tabs(window, panel.clone());
        let target = HoverTarget {
            tabs,
            root: tabs,
            leaf_tabs: tabs,
            zone: DropZone::Left,
            insert_index: None,
            outer: false,
            explicit: true,
        };
        let tx = resolve_dock_drop_transaction(
            DockDropTargetResolution {
                target: Some(DockDropTarget::Dock(target)),
                source: fret_runtime::DockDropResolveSource::InnerHintRect,
                policy: DockDropPolicyDecision::Allowed,
            },
            DockDropIntent::MovePanel {
                source_window: window,
                panel,
                target_window: window,
                target_tabs: tabs,
                zone: DropZone::Left,
                insert_index: None,
            },
        );

        let diag = super::super::diagnostics::compute_dock_drop_resolve_diagnostics(
            fret_core::PointerId(1),
            Point::new(Px(10.0), Px(10.0)),
            diagnostics_rect(),
            diagnostics_rect(),
            &graph,
            window,
            &tx,
            Vec::new(),
        );

        assert_eq!(
            diag.source,
            fret_runtime::DockDropResolveSource::InnerHintRect
        );
        assert_eq!(
            diag.policy,
            fret_runtime::DockDropPolicyDecisionDiagnostics::Allowed
        );
        assert_eq!(
            diag.command,
            fret_runtime::DockDropCommandKindDiagnostics::MovePanel
        );
        assert!(diag.commit_capable);
        assert!(diag.clears_hover);
        assert!(diag.invalidates_layout);
        assert_eq!(diag.resolved.map(|r| r.zone), Some(DropZone::Left));
        assert!(diag.denied.is_none());
        assert!(diag.preview.is_some());
    }

    #[test]
    fn transaction_diagnostics_preserve_policy_denial_reason_and_target() {
        let window = window(1);
        let panel = panel("demo.panel");
        let (graph, tabs) = graph_with_tabs(window, panel);
        let denied = HoverTarget {
            tabs,
            root: tabs,
            leaf_tabs: tabs,
            zone: DropZone::Bottom,
            insert_index: None,
            outer: true,
            explicit: true,
        };
        let tx = resolve_dock_drop_transaction(
            DockDropTargetResolution {
                target: None,
                source: fret_runtime::DockDropResolveSource::OuterHintRect,
                policy: DockDropPolicyDecision::Denied { target: denied },
            },
            DockDropIntent::None,
        );

        let diag = super::super::diagnostics::compute_dock_drop_resolve_diagnostics(
            fret_core::PointerId(1),
            Point::new(Px(10.0), Px(10.0)),
            diagnostics_rect(),
            diagnostics_rect(),
            &graph,
            window,
            &tx,
            Vec::new(),
        );

        assert_eq!(
            diag.policy,
            fret_runtime::DockDropPolicyDecisionDiagnostics::DeniedDockingPolicy
        );
        assert_eq!(
            diag.command,
            fret_runtime::DockDropCommandKindDiagnostics::None
        );
        assert!(!diag.commit_capable);
        assert!(diag.resolved.is_none());
        assert_eq!(diag.denied.map(|d| d.zone), Some(DropZone::Bottom));
    }
}
