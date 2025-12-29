// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude::*;

pub(super) fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    compute_layout_map_impl(graph, root, bounds, &mut layout);
    layout
}

pub(super) fn compute_layout_map_impl(
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
            let count = children.len().min(fractions.len());
            if count == 0 {
                return;
            }

            let total: f32 = fractions.iter().take(count).sum();
            let total = if total <= 0.0 { 1.0 } else { total };

            let axis_len = match axis {
                fret_core::Axis::Horizontal => bounds.size.width.0,
                fret_core::Axis::Vertical => bounds.size.height.0,
            };
            if !axis_len.is_finite() || axis_len <= 0.0 {
                return;
            }

            let gaps = count.saturating_sub(1) as f32;
            let mut gap = DOCK_SPLIT_HANDLE_GAP.0;
            if gaps == 0.0 || axis_len <= gap * gaps {
                gap = 0.0;
            }

            let available = axis_len - gap * gaps;
            if !available.is_finite() || available <= 0.0 {
                return;
            }

            let mut cursor = 0.0;
            for i in 0..count {
                let f = (fractions[i] / total).max(0.0);
                let (child_axis_len, next_cursor) = if i + 1 == count {
                    let remaining = (available - cursor).max(0.0);
                    (remaining, available)
                } else {
                    let len = available * f;
                    (len, cursor + len)
                };

                let origin_axis = cursor + gap * (i as f32);
                let child_rect = match axis {
                    fret_core::Axis::Horizontal => Rect {
                        origin: Point::new(Px(bounds.origin.x.0 + origin_axis), bounds.origin.y),
                        size: Size::new(Px(child_axis_len), bounds.size.height),
                    },
                    fret_core::Axis::Vertical => Rect {
                        origin: Point::new(bounds.origin.x, Px(bounds.origin.y.0 + origin_axis)),
                        size: Size::new(bounds.size.width, Px(child_axis_len)),
                    },
                };

                cursor = next_cursor;
                compute_layout_map_impl(graph, children[i], child_rect, out);
            }
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

pub(super) fn split_gap(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = match axis {
        fret_core::Axis::Horizontal => second.origin.x.0 - (first.origin.x.0 + first.size.width.0),
        fret_core::Axis::Vertical => second.origin.y.0 - (first.origin.y.0 + first.size.height.0),
    };
    if gap.is_finite() { gap.max(0.0) } else { 0.0 }
}

pub(super) fn split_handle_center(axis: fret_core::Axis, first: Rect, second: Rect) -> f32 {
    let gap = split_gap(axis, first, second);
    match axis {
        fret_core::Axis::Horizontal => {
            let start = first.origin.x.0 + first.size.width.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
        fret_core::Axis::Vertical => {
            let start = first.origin.y.0 + first.size.height.0;
            if gap > 0.0 { start + gap * 0.5 } else { start }
        }
    }
}

pub(super) fn split_handle_rect(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    thickness: Px,
) -> Rect {
    let gap = split_gap(axis, first, second);
    if gap > 0.0 {
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(first.origin.x.0 + first.size.width.0), bounds.origin.y),
                size: Size::new(Px(gap), bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(first.origin.y.0 + first.size.height.0)),
                size: Size::new(bounds.size.width, Px(gap)),
            },
        }
    } else {
        let center = split_handle_center(axis, first, second);
        match axis {
            fret_core::Axis::Horizontal => Rect {
                origin: Point::new(Px(center - thickness.0 * 0.5), bounds.origin.y),
                size: Size::new(thickness, bounds.size.height),
            },
            fret_core::Axis::Vertical => Rect {
                origin: Point::new(bounds.origin.x, Px(center - thickness.0 * 0.5)),
                size: Size::new(bounds.size.width, thickness),
            },
        }
    }
}

pub(super) fn compute_split_fraction(
    axis: fret_core::Axis,
    bounds: Rect,
    first: Rect,
    second: Rect,
    grab_offset: f32,
    position: Point,
) -> Option<f32> {
    let min_px = 120.0;
    match axis {
        fret_core::Axis::Horizontal => {
            let w = bounds.size.width.0;
            if !w.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = w - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_x = (avail - min_px).max(min_px);
            let anchor = position.x.0 - grab_offset - bounds.origin.x.0;
            let x = (anchor - gap * 0.5).clamp(min_px, max_x);
            Some(x / avail)
        }
        fret_core::Axis::Vertical => {
            let h = bounds.size.height.0;
            if !h.is_finite() {
                return None;
            }
            let gap = split_gap(axis, first, second);
            let avail = h - gap;
            if !avail.is_finite() || avail <= min_px * 2.0 {
                return None;
            }
            let max_y = (avail - min_px).max(min_px);
            let anchor = position.y.0 - grab_offset - bounds.origin.y.0;
            let y = (anchor - gap * 0.5).clamp(min_px, max_y);
            Some(y / avail)
        }
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
