// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

mod floating_hit;
mod target;

use super::prelude_core::*;
use super::prelude_runtime::*;
use super::types::DockDropTarget;
pub(super) use target::resolve_dock_drop_target;

#[derive(Clone, Copy)]
pub(super) struct DockPanelDropDrag<'a> {
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) panel: &'a PanelKey,
    pub(super) grab_offset: Point,
    pub(super) tear_off_requested: bool,
}

#[derive(Clone, Copy)]
pub(super) struct DockTabsDropDrag<'a> {
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) source_tabs: DockNodeId,
    pub(super) tabs: &'a [PanelKey],
    pub(super) active: usize,
    pub(super) grab_offset: Point,
    pub(super) tear_off_requested: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_dock_drop_intent_panel<F>(
    target: Option<DockDropTarget>,
    drag: DockPanelDropDrag<'_>,
    target_window: fret_core::AppWindowId,
    window_bounds: Rect,
    position: Point,
    allow_tear_off: bool,
    mark_drag_tear_off_requested: bool,
    default_floating_rect_for_panel: F,
) -> DockDropIntent
where
    F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
{
    match target {
        Some(DockDropTarget::Dock(target)) => DockDropIntent::MovePanel {
            source_window: drag.source_window,
            panel: drag.panel.clone(),
            target_window,
            target_tabs: target.tabs,
            zone: target.zone,
            insert_index: target.insert_index,
        },
        Some(DockDropTarget::EmptyDockSpace { .. }) => DockDropIntent::MovePanelToEmptyDockSpace {
            source_window: drag.source_window,
            panel: drag.panel.clone(),
            target_window,
        },
        Some(DockDropTarget::Float { .. }) => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatPanelToNewWindow {
                        source_window: drag.source_window,
                        panel: drag.panel.clone(),
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                let rect = default_floating_rect_for_panel(
                    drag.panel,
                    position,
                    drag.grab_offset,
                    window_bounds,
                );
                DockDropIntent::FloatPanelInWindow {
                    source_window: drag.source_window,
                    panel: drag.panel.clone(),
                    target_window,
                    rect,
                }
            }
        }
        None => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatPanelToNewWindow {
                        source_window: drag.source_window,
                        panel: drag.panel.clone(),
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                DockDropIntent::None
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_dock_drop_intent_tabs<F>(
    target: Option<DockDropTarget>,
    drag: DockTabsDropDrag<'_>,
    target_window: fret_core::AppWindowId,
    window_bounds: Rect,
    position: Point,
    allow_tear_off: bool,
    mark_drag_tear_off_requested: bool,
    default_floating_rect_for_panel: F,
) -> DockDropIntent
where
    F: FnOnce(&PanelKey, Point, Point, Rect) -> Rect,
{
    match target {
        Some(DockDropTarget::Dock(target)) => DockDropIntent::MoveTabs {
            source_window: drag.source_window,
            source_tabs: drag.source_tabs,
            target_window,
            target_tabs: target.tabs,
            zone: target.zone,
            insert_index: target.insert_index,
        },
        Some(DockDropTarget::EmptyDockSpace { .. }) => DockDropIntent::MoveTabsToEmptyDockSpace {
            source_window: drag.source_window,
            source_tabs: drag.source_tabs,
            target_window,
        },
        Some(DockDropTarget::Float { .. }) => {
            let wants_tear_off = allow_tear_off && !window_bounds.contains(position);
            let panel = drag
                .tabs
                .get(drag.active)
                .or_else(|| drag.tabs.first())
                .cloned();
            let Some(panel) = panel else {
                return DockDropIntent::None;
            };
            if wants_tear_off {
                if drag.tear_off_requested || mark_drag_tear_off_requested {
                    DockDropIntent::None
                } else {
                    DockDropIntent::RequestFloatTabsToNewWindow {
                        source_window: drag.source_window,
                        source_tabs: drag.source_tabs,
                        panel,
                        anchor: Some(fret_core::WindowAnchor {
                            window: target_window,
                            position: drag.grab_offset,
                        }),
                    }
                }
            } else {
                let rect = default_floating_rect_for_panel(
                    &panel,
                    position,
                    drag.grab_offset,
                    window_bounds,
                );
                DockDropIntent::FloatTabsInWindow {
                    source_window: drag.source_window,
                    source_tabs: drag.source_tabs,
                    target_window,
                    rect,
                }
            }
        }
        None => DockDropIntent::None,
    }
}

pub(super) fn apply_dock_drop_intent(
    intent: DockDropIntent,
    pending_effects: &mut Vec<Effect>,
    invalidate_layout: &mut bool,
) {
    match intent {
        DockDropIntent::None => {}
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
            *invalidate_layout = true;
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
            *invalidate_layout = true;
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
            *invalidate_layout = true;
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
            *invalidate_layout = true;
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
            *invalidate_layout = true;
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
            *invalidate_layout = true;
        }
        DockDropIntent::RequestFloatPanelToNewWindow {
            source_window,
            panel,
            anchor,
        } => {
            pending_effects.push(Effect::Dock(DockOp::RequestFloatPanelToNewWindow {
                source_window,
                panel,
                anchor,
            }));
            *invalidate_layout = true;
        }
        DockDropIntent::RequestFloatTabsToNewWindow {
            source_window,
            source_tabs,
            panel,
            anchor,
        } => {
            pending_effects.push(Effect::Dock(DockOp::RequestFloatTabsToNewWindow {
                source_window,
                source_tabs,
                panel,
                anchor,
            }));
            *invalidate_layout = true;
        }
    }
}

pub(super) fn dock_drop_intent_debug_kind(intent: &DockDropIntent) -> &'static str {
    match intent {
        DockDropIntent::None => "none",
        DockDropIntent::MovePanel { .. } => "move_panel",
        DockDropIntent::MovePanelToEmptyDockSpace { .. } => "move_panel_to_empty_dock_space",
        DockDropIntent::MoveTabs { .. } => "move_tabs",
        DockDropIntent::MoveTabsToEmptyDockSpace { .. } => "move_tabs_to_empty_dock_space",
        DockDropIntent::FloatPanelInWindow { .. } => "float_panel_in_window",
        DockDropIntent::FloatTabsInWindow { .. } => "float_tabs_in_window",
        DockDropIntent::RequestFloatPanelToNewWindow { .. } => "request_float_panel_to_new_window",
        DockDropIntent::RequestFloatTabsToNewWindow { .. } => "request_float_tabs_to_new_window",
    }
}

pub(super) fn dock_drop_target_diagnostics(
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
pub(super) fn compute_dock_drop_resolve_diagnostics(
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
