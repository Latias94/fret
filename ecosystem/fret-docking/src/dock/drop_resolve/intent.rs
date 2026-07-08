// This file is part of the docking UI implementation.
//
// It owns pure drop intent projection for resolved dock drop targets.

use super::super::prelude_core::*;
use super::super::types::{DockDropIntent, DockDropTarget};

#[derive(Clone, Copy)]
pub(in crate::dock) struct DockPanelDropDrag<'a> {
    pub(in crate::dock) source_window: fret_core::AppWindowId,
    pub(in crate::dock) panel: &'a PanelKey,
    pub(in crate::dock) grab_offset: Point,
    pub(in crate::dock) tear_off_requested: bool,
}

#[derive(Clone, Copy)]
pub(in crate::dock) struct DockTabsDropDrag<'a> {
    pub(in crate::dock) source_window: fret_core::AppWindowId,
    pub(in crate::dock) source_tabs: DockNodeId,
    pub(in crate::dock) tabs: &'a [PanelKey],
    pub(in crate::dock) active: usize,
    pub(in crate::dock) grab_offset: Point,
    pub(in crate::dock) tear_off_requested: bool,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::dock) fn resolve_dock_drop_intent_panel<F>(
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
pub(in crate::dock) fn resolve_dock_drop_intent_tabs<F>(
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
