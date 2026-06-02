use std::collections::HashMap;

use fret_core::{AppWindowId, PanelKey, Rect, Size};
use fret_ui::UiHost;

use super::super::super::drop_resolve::{
    DockPanelDropDrag, DockTabsDropDrag, resolve_dock_drop_intent_panel,
    resolve_dock_drop_intent_tabs,
};
use super::super::super::types::{
    DockDropIntent, DockDropTarget, DockPanelDragPayload, DockTabsDragPayload,
};
use super::super::tear_off::{
    declarative_allow_tear_off_for_panel, declarative_default_floating_rect_for_panel,
};

// This file owns declarative docking drop-intent preparation for panel and tab drags.

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_declarative_drag_drop_intent<H: UiHost>(
    app: &H,
    target: Option<&DockDropTarget>,
    panel_payload: Option<&DockPanelDragPayload>,
    tabs_payload: Option<&DockTabsDragPayload>,
    source_window: AppWindowId,
    window: AppWindowId,
    bounds: Rect,
    position: fret_core::Point,
    allow_tear_off: bool,
    allow_multi_window_tear_off: bool,
    paint_panel_bounds: &[(PanelKey, Rect)],
) -> DockDropIntent {
    let panel_last_sizes: HashMap<PanelKey, Size> = paint_panel_bounds
        .iter()
        .map(|(panel, rect)| (panel.clone(), rect.size))
        .collect();
    if let Some(payload) = panel_payload {
        let allow_panel_tear_off = declarative_allow_tear_off_for_panel(
            app,
            allow_tear_off,
            allow_multi_window_tear_off,
            source_window,
            &payload.panel,
        );
        return resolve_dock_drop_intent_panel(
            target.cloned(),
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
        );
    }

    if let Some(payload) = tabs_payload {
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
        return resolve_dock_drop_intent_tabs(
            target.cloned(),
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
        );
    }

    DockDropIntent::None
}
