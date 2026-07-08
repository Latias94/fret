use std::collections::HashMap;

use fret_core::{AppWindowId, Rect};
use fret_runtime::Effect;
use fret_ui::UiHost;
use fret_ui_headless::tab_strip_controller as tabstrip_controller;

use super::super::hit_test::hit_test_tab;
use super::super::manager::DockManager;
use super::super::tab_overflow::{
    TabOverflowMenuState, compute_tab_overflow_menu_items, overflow_menu_close_rect,
    overflow_menu_max_scroll, overflow_menu_row_at_pos, overflow_menu_row_count,
    overflow_menu_row_height, overflow_menu_row_rect, tab_overflow_button_rect,
    tab_overflow_menu_rect,
};
use super::interaction::{DeclarativeDockInteractionService, DeclarativeTabHover};
use super::tab_metrics::{
    declarative_clamp_and_ensure_active_visible, declarative_tab_bar_geometry,
    declarative_tab_scroll_for_frame, declarative_tab_widths_for_layout,
};

pub(super) fn declarative_tab_overflow_menu_for_window<H: UiHost>(
    app: &H,
    window: AppWindowId,
) -> Option<TabOverflowMenuState> {
    app.global::<DeclarativeDockInteractionService>()
        .and_then(|service| service.tab_overflow_menu(window))
}

pub(super) fn declarative_open_tab_overflow_menu<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    tab_scroll: &HashMap<fret_core::DockNodeId, fret_core::Px>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> Option<TabOverflowMenuState> {
    let dock = app.global::<DockManager>()?;
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, active }) = dock.workspace.graph.node(node_id)
        else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let Some(widths) = tab_widths.get(&node_id) else {
            continue;
        };
        let (geom, _overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        let max_scroll = geom.max_scroll();
        if max_scroll.0 <= 0.0 {
            continue;
        }
        if !tab_overflow_button_rect(theme.clone(), tab_bar).contains(position) {
            continue;
        }

        let items = compute_tab_overflow_menu_items(
            theme.clone(),
            tab_bar,
            tabs.len(),
            Some(widths),
            tab_scroll
                .get(&node_id)
                .copied()
                .unwrap_or(fret_core::Px(0.0)),
            *active,
        );
        if items.is_empty() {
            continue;
        }
        let item_count = items.len();
        let active_row = items.iter().position(|ix| *ix == *active).unwrap_or(0);
        let row_h = overflow_menu_row_height(tab_bar).0;
        let visible = overflow_menu_row_count(item_count) as f32;
        let active_y = active_row as f32 * row_h;
        let min_scroll = active_y - (visible - 1.0) * row_h;
        let max_scroll_menu = overflow_menu_max_scroll(tab_bar, item_count);
        let scroll = fret_core::Px(min_scroll.clamp(0.0, max_scroll_menu.0.max(0.0)));
        return Some(TabOverflowMenuState {
            tabs: node_id,
            items,
            scroll,
            hovered: None,
        });
    }
    None
}

pub(super) fn declarative_handle_tab_overflow_menu_left_click<H: UiHost>(
    app: &H,
    window: AppWindowId,
    menu: TabOverflowMenuState,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> (bool, Option<TabOverflowMenuState>, Vec<Effect>) {
    let Some(dock) = app.global::<DockManager>() else {
        return (false, None, Vec::new());
    };
    let mut keep_open = true;
    let mut handled = false;
    let mut effects = Vec::new();

    let tabs_rect = layout_all.get(&menu.tabs).copied();
    let node = dock.workspace.graph.node(menu.tabs);
    if let (Some(tabs_rect), Some(fret_core::DockNode::Tabs { tabs, .. })) = (tabs_rect, node) {
        let (tab_bar, _content) = super::super::layout::split_tab_bar(tabs_rect);
        let item_count = menu.items.len();
        let menu_rect = tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
        let button_rect = tab_overflow_button_rect(theme.clone(), tab_bar);

        if menu_rect.contains(position) {
            handled = true;
            let max_scroll = overflow_menu_max_scroll(tab_bar, item_count);
            let scroll = fret_core::Px(menu.scroll.0.clamp(0.0, max_scroll.0));
            let row = overflow_menu_row_at_pos(menu_rect, tab_bar, item_count, scroll, position);
            if let Some(row) = row
                && let Some(&tab_ix) = menu.items.get(row)
            {
                let row_rect = overflow_menu_row_rect(menu_rect, tab_bar, scroll, row);
                let close_rect = overflow_menu_close_rect(theme.clone(), row_rect);
                let hit = if close_rect.contains(position) {
                    tabstrip_controller::TabStripHitTarget::OverflowMenuRow {
                        index: tab_ix,
                        part: tabstrip_controller::OverflowMenuPart::Close,
                    }
                } else {
                    tabstrip_controller::TabStripHitTarget::OverflowMenuRow {
                        index: tab_ix,
                        part: tabstrip_controller::OverflowMenuPart::Content,
                    }
                };

                match tabstrip_controller::intent_for_click(hit) {
                    tabstrip_controller::TabStripIntent::Close { index } => {
                        if let Some(panel) = tabs.get(index) {
                            effects.push(Effect::Dock(fret_core::DockOp::ClosePanel {
                                window,
                                panel: panel.clone(),
                            }));
                            keep_open = false;
                        }
                    }
                    tabstrip_controller::TabStripIntent::Activate { index, .. } => {
                        effects.push(Effect::Dock(fret_core::DockOp::SetActiveTab {
                            tabs: menu.tabs,
                            active: index,
                        }));
                        keep_open = false;
                    }
                    tabstrip_controller::TabStripIntent::ToggleOverflowMenu
                    | tabstrip_controller::TabStripIntent::None => {}
                }
            }
        } else if button_rect.contains(position) {
            keep_open = false;
            handled = true;
        } else {
            keep_open = false;
        }
    } else {
        keep_open = false;
    }

    let next_menu = keep_open.then_some(menu);
    (handled, next_menu, effects)
}

pub(super) fn declarative_handle_tab_overflow_menu_wheel<H: UiHost>(
    app: &H,
    menu: TabOverflowMenuState,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    delta: fret_core::Point,
) -> (bool, Option<TabOverflowMenuState>) {
    let Some(dock) = app.global::<DockManager>() else {
        return (false, Some(menu));
    };
    let Some(tabs_rect) = layout_all.get(&menu.tabs).copied() else {
        return (false, Some(menu));
    };
    let Some(fret_core::DockNode::Tabs { .. }) = dock.workspace.graph.node(menu.tabs) else {
        return (false, Some(menu));
    };

    let (tab_bar, _content) = super::super::layout::split_tab_bar(tabs_rect);
    let item_count = menu.items.len();
    if item_count == 0 {
        return (true, None);
    }

    let menu_rect = tab_overflow_menu_rect(theme, tab_bar, item_count);
    if !menu_rect.contains(position) {
        return (false, Some(menu));
    }

    let max_scroll = overflow_menu_max_scroll(tab_bar, item_count);
    let wheel = delta.x.0 + delta.y.0;
    let next_scroll = fret_core::Px((menu.scroll.0 - wheel).clamp(0.0, max_scroll.0));
    let hovered = overflow_menu_row_at_pos(menu_rect, tab_bar, item_count, next_scroll, position);
    let next_menu = TabOverflowMenuState {
        scroll: next_scroll,
        hovered,
        ..menu
    };
    (true, Some(next_menu))
}

pub(super) fn declarative_handle_tab_strip_wheel<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
    delta: fret_core::Point,
) -> Option<HashMap<fret_core::DockNodeId, fret_core::Px>> {
    let dock = app.global::<DockManager>()?;
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    let mut tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        layout_all,
        &tab_widths,
        false,
    );

    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, active }) = dock.workspace.graph.node(node_id)
        else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }

        declarative_clamp_and_ensure_active_visible(
            &mut tab_scroll,
            &tab_widths,
            theme.clone(),
            node_id,
            tab_bar,
            tabs.len(),
            *active,
        );
        let (geom, _overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        let max_scroll = geom.max_scroll();
        if max_scroll.0 <= 0.0 {
            return Some(tab_scroll);
        }

        let wheel = delta.x.0 + delta.y.0;
        let scroll = tab_scroll
            .get(&node_id)
            .copied()
            .unwrap_or(fret_core::Px(0.0));
        let next = fret_core::Px((scroll.0 - wheel).clamp(0.0, max_scroll.0));
        if next.0 <= 0.0 {
            tab_scroll.remove(&node_id);
        } else {
            tab_scroll.insert(node_id, next);
        }
        return Some(tab_scroll);
    }

    None
}

pub(super) fn declarative_tab_hover_for_position<H: UiHost>(
    app: &H,
    window: AppWindowId,
    layout_all: &HashMap<fret_core::DockNodeId, Rect>,
    theme: fret_ui::ThemeSnapshot,
    position: fret_core::Point,
) -> (DeclarativeTabHover, Option<TabOverflowMenuState>, bool) {
    let Some(dock) = app.global::<DockManager>() else {
        return (DeclarativeTabHover::default(), None, false);
    };
    let tab_widths = declarative_tab_widths_for_layout(app, window, theme.clone(), layout_all);
    let tab_scroll = declarative_tab_scroll_for_frame(
        app,
        window,
        theme.clone(),
        layout_all,
        &tab_widths,
        false,
    );

    let hovered = hit_test_tab(
        &dock.workspace.graph,
        layout_all,
        &tab_scroll,
        &tab_widths,
        theme.clone(),
        position,
    )
    .map(|(node, idx, _panel, close)| (node, idx, close));
    let mut hover = DeclarativeTabHover {
        tab: hovered.map(|(node, idx, _close)| (node, idx)),
        tab_close: hovered.map(|(_node, _idx, close)| close).unwrap_or(false),
        overflow_button: None,
    };
    let mut pointer_cursor = hover.tab.is_some();

    for (&node_id, &rect) in layout_all {
        let Some(fret_core::DockNode::Tabs { tabs, .. }) = dock.workspace.graph.node(node_id)
        else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = super::super::layout::split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let (_geom, overflow) =
            declarative_tab_bar_geometry(theme.clone(), &tab_widths, node_id, tab_bar, tabs.len());
        if overflow && tab_overflow_button_rect(theme.clone(), tab_bar).contains(position) {
            hover.overflow_button = Some(node_id);
            pointer_cursor = true;
            break;
        }
    }

    let mut next_menu = declarative_tab_overflow_menu_for_window(app, window);
    if let Some(menu) = next_menu.as_mut() {
        let mut close_menu = false;
        if let Some(&tabs_rect) = layout_all.get(&menu.tabs) {
            if dock.workspace.graph.node(menu.tabs).is_some() {
                let (tab_bar, _content) = super::super::layout::split_tab_bar(tabs_rect);
                let item_count = menu.items.len();
                if item_count == 0 {
                    close_menu = true;
                } else {
                    let menu_rect = tab_overflow_menu_rect(theme.clone(), tab_bar, item_count);
                    menu.hovered = if menu_rect.contains(position) {
                        pointer_cursor = true;
                        overflow_menu_row_at_pos(
                            menu_rect,
                            tab_bar,
                            item_count,
                            menu.scroll,
                            position,
                        )
                    } else {
                        None
                    };
                }
            } else {
                close_menu = true;
            }
        } else {
            close_menu = true;
        }
        if close_menu {
            next_menu = None;
        }
    }

    (hover, next_menu, pointer_cursor)
}
