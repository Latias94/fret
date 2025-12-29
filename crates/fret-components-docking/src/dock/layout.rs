// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

pub(super) fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    compute_layout_map_impl(graph, root, bounds, &mut layout);
    layout
}

fn compute_layout_map_impl(
    graph: &DockGraph,
    node: DockNodeId,
    bounds: Rect,
    out: &mut std::collections::HashMap<DockNodeId, Rect>,
) {
    let Some(n) = graph.node(node) else {
        return;
    };

    out.insert(node, bounds);
    match n {
        DockNode::Tabs { .. } => {}
        DockNode::Split {
            axis,
            children,
            fractions,
        } => {
            let count = children.len();
            if count == 0 {
                return;
            }
            let computed = resizable::compute_layout(
                *axis,
                bounds,
                count,
                fractions,
                DOCK_SPLIT_HANDLE_GAP,
                DOCK_SPLIT_HANDLE_HIT_THICKNESS,
                &[],
            );
            for (&child, &rect) in children.iter().zip(computed.panel_rects.iter()) {
                compute_layout_map_impl(graph, child, rect, out);
            }
        }
        DockNode::Floating { child } => {
            compute_layout_map_impl(graph, *child, bounds, out);
        }
    }
}

pub(super) fn hidden_bounds(size: Size) -> Rect {
    Rect {
        origin: Point::new(Px(-1_000_000.0), Px(-1_000_000.0)),
        size,
    }
}

pub(super) fn active_panel_content_bounds(
    graph: &DockGraph,
    layout: &std::collections::HashMap<DockNodeId, Rect>,
) -> std::collections::HashMap<PanelKey, Rect> {
    let mut out: std::collections::HashMap<PanelKey, Rect> = std::collections::HashMap::new();

    for (&node_id, &rect) in layout.iter() {
        let Some(DockNode::Tabs { tabs, active }) = graph.node(node_id) else {
            continue;
        };
        let (_tab_bar, content) = split_tab_bar(rect);
        if let Some(panel) = tabs.get(*active) {
            out.insert(panel.clone(), content);
        }
    }

    out
}

pub(super) fn split_tab_bar(rect: Rect) -> (Rect, Rect) {
    let tab_bar = Rect {
        origin: rect.origin,
        size: Size::new(rect.size.width, Px(DOCK_TAB_H.0.min(rect.size.height.0))),
    };
    let content = Rect {
        origin: Point::new(rect.origin.x, Px(rect.origin.y.0 + tab_bar.size.height.0)),
        size: Size::new(
            rect.size.width,
            Px((rect.size.height.0 - tab_bar.size.height.0).max(0.0)),
        ),
    };
    (tab_bar, content)
}

pub(super) fn dock_drop_edge_thickness(rect: Rect) -> Px {
    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Keep split zones usable on large panels, but avoid making "center tab" drops difficult.
    // Also keep the thickness sane on small panels.
    // ImGui-style: edge splits should be easy to hit even on big panels; we still cap it so the
    // center/tab drop remains a first-class target.
    let base = (min_dim * 0.30).clamp(20.0, 120.0);
    let cap = (min_dim * 0.44).clamp(20.0, 120.0);
    Px(base.min(cap))
}

pub(super) fn drop_zone_rect(rect: Rect, zone: DropZone) -> Rect {
    if zone == DropZone::Center {
        return rect;
    }
    let thickness = dock_drop_edge_thickness(rect).0;
    match zone {
        DropZone::Left => Rect {
            origin: rect.origin,
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Right => Rect {
            origin: Point::new(
                Px(rect.origin.x.0 + rect.size.width.0 - thickness),
                rect.origin.y,
            ),
            size: Size::new(Px(thickness), rect.size.height),
        },
        DropZone::Top => Rect {
            origin: rect.origin,
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Bottom => Rect {
            origin: Point::new(
                rect.origin.x,
                Px(rect.origin.y.0 + rect.size.height.0 - thickness),
            ),
            size: Size::new(rect.size.width, Px(thickness)),
        },
        DropZone::Center => rect,
    }
}

pub(super) fn float_zone(bounds: Rect) -> Rect {
    let size = Px(34.0);
    Rect {
        origin: Point::new(Px(bounds.origin.x.0 + 8.0), Px(bounds.origin.y.0 + 8.0)),
        size: Size::new(size, size),
    }
}

pub(super) fn dock_hint_rects(rect: Rect) -> [(DropZone, Rect); 5] {
    // Match the mental model of ImGui docking: an explicit 5-way “direction pad” near the
    // center of the hovered dock node. Hit-testing uses the same rects.
    let cx = rect.origin.x.0 + rect.size.width.0 * 0.5;
    let cy = rect.origin.y.0 + rect.size.height.0 * 0.5;

    let min_dim = rect.size.width.0.min(rect.size.height.0);
    // Scale targets up on larger panels to make split docking feel effortless (Unity/ImGui-like),
    // while keeping it usable on small panels.
    let size = Px((min_dim * 0.095).clamp(34.0, 56.0));
    let gap = Px((size.0 * 0.35).clamp(10.0, 16.0));
    let step = Px(size.0 + gap.0);

    let mk = |dx: f32, dy: f32| -> Rect {
        Rect::new(
            Point::new(Px(cx + dx - size.0 * 0.5), Px(cy + dy - size.0 * 0.5)),
            Size::new(size, size),
        )
    };

    [
        (DropZone::Center, mk(0.0, 0.0)),
        (DropZone::Left, mk(-step.0, 0.0)),
        (DropZone::Right, mk(step.0, 0.0)),
        (DropZone::Top, mk(0.0, -step.0)),
        (DropZone::Bottom, mk(0.0, step.0)),
    ]
}

pub(super) fn dock_space_regions(bounds: Rect) -> (Rect, Rect) {
    let chrome_h = Px(0.0);
    let chrome = Rect {
        origin: bounds.origin,
        size: Size::new(bounds.size.width, Px(chrome_h.0.min(bounds.size.height.0))),
    };
    let dock = Rect {
        origin: Point::new(
            bounds.origin.x,
            Px(bounds.origin.y.0 + chrome.size.height.0),
        ),
        size: Size::new(
            bounds.size.width,
            Px((bounds.size.height.0 - chrome.size.height.0).max(0.0)),
        ),
    };
    (chrome, dock)
}
