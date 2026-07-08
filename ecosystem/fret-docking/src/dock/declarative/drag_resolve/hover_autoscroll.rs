use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{AppWindowId, DockNodeId, Point, Px, Rect};
use fret_ui::{ThemeSnapshot, UiHost};

use super::super::super::layout::split_tab_bar;
use super::super::super::manager::DockManager;
use super::super::super::types::DockDropTarget;
use super::super::interaction::DeclarativeDockInteractionService;
use super::super::tab_metrics::{
    declarative_apply_tab_bar_drag_auto_scroll, declarative_sync_tab_scroll_for_window,
};

// This file owns declarative docking hover-time tab-bar drag auto-scroll.

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_drag_hover_auto_scroll<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    hover: &mut Option<DockDropTarget>,
    layout_all: &HashMap<DockNodeId, Rect>,
    theme: ThemeSnapshot,
    font_size: Px,
    position: Point,
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
    tab_scroll: &mut HashMap<DockNodeId, Px>,
    dragged_tab_for_drop: Option<(DockNodeId, usize)>,
) -> bool {
    let Some(DockDropTarget::Dock(target)) = hover.as_mut() else {
        return false;
    };
    let target_tabs = target.tabs;
    let tabs_len =
        app.global::<DockManager>()
            .and_then(|dock| match dock.workspace.graph.node(target_tabs) {
                Some(fret_core::DockNode::Tabs { tabs, .. }) => Some(tabs.len()),
                _ => None,
            });
    let tabs_rect = layout_all.get(&target_tabs).copied();
    let frame_id = app.frame_id();
    let should_scroll = tabs_len.is_some()
        && tabs_rect.is_some()
        && app.with_global_mut(
            DeclarativeDockInteractionService::default,
            |service, _app| service.should_auto_scroll_tab_drag(window, target_tabs, frame_id),
        );
    let (true, Some(tabs_len), Some(tabs_rect)) = (should_scroll, tabs_len, tabs_rect) else {
        return false;
    };

    let (tab_bar, _content) = split_tab_bar(tabs_rect);
    let auto_scrolled = declarative_apply_tab_bar_drag_auto_scroll(
        theme,
        target,
        tab_bar,
        tabs_len,
        font_size,
        position,
        tab_widths,
        tab_scroll,
        dragged_tab_for_drop,
    );
    if auto_scrolled {
        declarative_sync_tab_scroll_for_window(app, window, tab_scroll, layout_all.keys().copied());
    }
    auto_scrolled
}
