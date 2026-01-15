// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::layout::split_tab_bar;
#[cfg(test)]
use super::layout::{dock_drop_edge_thickness, dock_hint_rects};
use super::prelude_core::*;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

pub(super) fn tab_scroll_for_node(tab_scroll: &HashMap<DockNodeId, Px>, node: DockNodeId) -> Px {
    tab_scroll.get(&node).copied().unwrap_or(Px(0.0))
}

pub(super) fn tab_rect_for_index(tab_bar: Rect, index: usize, scroll: Px) -> Rect {
    Rect {
        origin: Point::new(
            Px(tab_bar.origin.x.0 + DOCK_TAB_W.0 * index as f32 - scroll.0),
            tab_bar.origin.y,
        ),
        size: Size::new(DOCK_TAB_W, tab_bar.size.height),
    }
}

pub(super) fn compute_tab_insert_index(
    tab_bar: Rect,
    scroll: Px,
    tab_count: usize,
    position: Point,
) -> usize {
    if tab_count == 0 {
        return 0;
    }

    let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
    if rel_x <= 0.0 {
        return 0;
    }

    let max_x = DOCK_TAB_W.0 * tab_count as f32;
    if rel_x >= max_x {
        return tab_count;
    }

    let over_index = (rel_x / DOCK_TAB_W.0).floor() as usize;
    let over_rect = tab_rect_for_index(tab_bar, over_index, scroll);
    let side = fret_dnd::insertion_side_for_pointer(position, over_rect, fret_dnd::Axis::X);
    over_index.saturating_add(match side {
        fret_dnd::InsertionSide::Before => 0,
        fret_dnd::InsertionSide::After => 1,
    })
}

pub(super) fn tab_close_rect(theme: fret_ui::ThemeSnapshot, tab_rect: Rect) -> Rect {
    let pad = theme.metric_required("metric.padding.sm").0.max(0.0);
    let x = tab_rect.origin.x.0 + tab_rect.size.width.0 - pad - DOCK_TAB_CLOSE_SIZE.0;
    let y = tab_rect.origin.y.0 + (tab_rect.size.height.0 - DOCK_TAB_CLOSE_SIZE.0) * 0.5;
    Rect::new(
        Point::new(Px(x), Px(y)),
        Size::new(DOCK_TAB_CLOSE_SIZE, DOCK_TAB_CLOSE_SIZE),
    )
}

pub(super) fn hit_test_tab(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    theme: fret_ui::ThemeSnapshot,
    position: Point,
) -> Option<(DockNodeId, usize, PanelKey, bool)> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if tabs.is_empty() {
            continue;
        }
        let (tab_bar, _content) = split_tab_bar(rect);
        if !tab_bar.contains(position) {
            continue;
        }
        let scroll = tab_scroll_for_node(tab_scroll, node);
        let rel_x = position.x.0 - tab_bar.origin.x.0 + scroll.0;
        let idx = (rel_x / DOCK_TAB_W.0).floor() as isize;
        if idx < 0 {
            continue;
        }
        let idx = idx as usize;
        let panel = tabs.get(idx)?.clone();
        let tab_rect = tab_rect_for_index(tab_bar, idx, scroll);
        let close = tab_close_rect(theme, tab_rect).contains(position);
        return Some((node, idx, panel, close));
    }
    None
}

#[cfg(test)]
pub(super) fn hit_test_drop_target(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    tab_scroll: &HashMap<DockNodeId, Px>,
    position: Point,
) -> Option<HoverTarget> {
    for (&node, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, .. }) = graph.node(node) else {
            continue;
        };
        if !rect.contains(position) {
            continue;
        }

        let (tab_bar, _content) = split_tab_bar(rect);
        if tab_bar.contains(position) {
            let scroll = tab_scroll_for_node(tab_scroll, node);
            let insert_index = compute_tab_insert_index(tab_bar, scroll, tabs.len(), position);
            return Some(HoverTarget {
                tabs: node,
                zone: DropZone::Center,
                insert_index: Some(insert_index),
            });
        }

        // ImGui-style direction-pad hit targets near the center of the hovered dock node.
        // This makes split docking discoverable and avoids requiring the cursor to be near edges.
        for (zone, hint_rect) in dock_hint_rects(rect) {
            if hint_rect.contains(position) {
                return Some(HoverTarget {
                    tabs: node,
                    zone,
                    insert_index: None,
                });
            }
        }

        let thickness = dock_drop_edge_thickness(rect).0;
        let left = position.x.0 - rect.origin.x.0;
        let right = rect.origin.x.0 + rect.size.width.0 - position.x.0;
        let top = position.y.0 - rect.origin.y.0;
        let bottom = rect.origin.y.0 + rect.size.height.0 - position.y.0;

        let mut zone = DropZone::Center;
        let mut best = thickness;
        for (candidate, dist) in [
            (DropZone::Left, left),
            (DropZone::Right, right),
            (DropZone::Top, top),
            (DropZone::Bottom, bottom),
        ] {
            if dist < best {
                best = dist;
                zone = candidate;
            }
        }

        return Some(HoverTarget {
            tabs: node,
            zone,
            insert_index: None,
        });
    }
    None
}

pub(super) fn hit_test_split_handle(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
    position: Point,
) -> Option<DividerDragState> {
    for (&node, &bounds) in layout.iter() {
        let Some(DockNode::Split {
            axis,
            children,
            fractions,
        }) = graph.node(node)
        else {
            continue;
        };
        if children.len() < 2 {
            continue;
        }
        if !bounds.contains(position) {
            continue;
        }

        let computed = resizable::compute_layout(
            *axis,
            bounds,
            children.len(),
            fractions,
            DOCK_SPLIT_HANDLE_GAP,
            DOCK_SPLIT_HANDLE_HIT_THICKNESS,
            &[],
        );
        for (handle_ix, rect) in computed.handle_hit_rects.iter().enumerate() {
            if !rect.contains(position) {
                continue;
            }
            let center = *computed.handle_centers.get(handle_ix).unwrap_or(&0.0);
            let grab_offset = match axis {
                fret_core::Axis::Horizontal => position.x.0 - center,
                fret_core::Axis::Vertical => position.y.0 - center,
            };
            return Some(DividerDragState {
                split: node,
                axis: *axis,
                bounds,
                handle_ix,
                grab_offset,
            });
        }
    }

    None
}
