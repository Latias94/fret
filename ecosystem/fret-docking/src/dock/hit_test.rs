// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

#[cfg(test)]
use super::layout::dock_hint_rects_with_font;
use super::layout::split_tab_bar;
use super::prelude_core::*;
use super::tab_bar_geometry::TabBarGeometry;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

pub(super) fn tab_scroll_for_node(tab_scroll: &HashMap<DockNodeId, Px>, node: DockNodeId) -> Px {
    tab_scroll.get(&node).copied().unwrap_or(Px(0.0))
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
    tab_widths: &HashMap<DockNodeId, Arc<[Px]>>,
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
        let geom = tab_widths
            .get(&node)
            .filter(|w| w.len() == tabs.len())
            .map(|w| TabBarGeometry::variable(tab_bar, w.clone()))
            .unwrap_or_else(|| TabBarGeometry::fixed(tab_bar, tabs.len()));
        let idx = geom.hit_test_tab_index(position, scroll)?;
        let panel = tabs.get(idx)?.clone();
        let tab_rect = geom.tab_rect(idx, scroll);
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
            let geom = TabBarGeometry::fixed(tab_bar, tabs.len());
            let insert_index = geom.compute_insert_index(position, scroll);
            return Some(HoverTarget {
                tabs: node,
                zone: DropZone::Center,
                insert_index: Some(insert_index),
            });
        }

        // ImGui-style direction-pad hit targets near the center of the hovered dock node.
        // This makes split docking discoverable and avoids requiring the cursor to be near edges.
        for (zone, hint_rect) in dock_hint_rects_with_font(rect, Px(13.0), false) {
            if hint_rect.contains(position) {
                return Some(HoverTarget {
                    tabs: node,
                    zone,
                    insert_index: None,
                });
            }
        }

        return Some(HoverTarget {
            tabs: node,
            zone: DropZone::Center,
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
