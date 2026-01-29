// This file is part of the docking UI implementation.
//
// It is intentionally `pub(super)` only; the public API lives in `dock/mod.rs`.

use super::prelude_core::*;
use fret_ui::retained_bridge::resizable_panel_group as resizable;

pub(super) fn compute_layout_map(
    graph: &DockGraph,
    root: DockNodeId,
    bounds: Rect,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
) -> std::collections::HashMap<DockNodeId, Rect> {
    let mut layout = std::collections::HashMap::new();
    compute_layout_map_impl(
        graph,
        root,
        bounds,
        split_handle_gap,
        split_handle_hit_thickness,
        &mut layout,
    );
    layout
}

fn compute_layout_map_impl(
    graph: &DockGraph,
    node: DockNodeId,
    bounds: Rect,
    split_handle_gap: Px,
    split_handle_hit_thickness: Px,
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
                split_handle_gap,
                split_handle_hit_thickness,
                &[],
            );
            for (&child, &rect) in children.iter().zip(computed.panel_rects.iter()) {
                compute_layout_map_impl(
                    graph,
                    child,
                    rect,
                    split_handle_gap,
                    split_handle_hit_thickness,
                    out,
                );
            }
        }
        DockNode::Floating { child } => {
            compute_layout_map_impl(
                graph,
                *child,
                bounds,
                split_handle_gap,
                split_handle_hit_thickness,
                out,
            );
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

pub(super) fn dock_hint_pick_zone(
    rect: Rect,
    font_size: Px,
    outer_docking: bool,
    position: Point,
) -> Option<DropZone> {
    // Align with Dear ImGui docking branch hit testing.
    //
    // Reference:
    // - `repo-ref/imgui/imgui.cpp`: `DockNodeCalcDropRectsAndTestMousePos(...)`
    let parent_smaller_axis = rect.size.width.0.min(rect.size.height.0);
    let font = font_size.0.max(0.0);
    let hs_for_central_nodes = (font * 1.5).min((font * 0.5).max(parent_smaller_axis / 8.0));

    let hs_w = if outer_docking {
        (hs_for_central_nodes * 1.50).trunc()
    } else {
        hs_for_central_nodes.trunc()
    };

    let cx = (rect.origin.x.0 + rect.size.width.0 * 0.5).trunc();
    let cy = (rect.origin.y.0 + rect.size.height.0 * 0.5).trunc();

    if !outer_docking {
        // Custom hit testing for the 5-way selection, designed to reduce flickering when moving
        // diagonally between sides.
        let dx = position.x.0 - cx;
        let dy = position.y.0 - cy;
        let len2 = dx * dx + dy * dy;
        let r_threshold_center = hs_w * 1.4;
        let r_threshold_sides = hs_w * (1.4 + 1.2);
        if len2 < r_threshold_center * r_threshold_center {
            return Some(DropZone::Center);
        }
        if len2 < r_threshold_sides * r_threshold_sides {
            return Some(if dx.abs() > dy.abs() {
                if dx > 0.0 {
                    DropZone::Right
                } else {
                    DropZone::Left
                }
            } else if dy > 0.0 {
                DropZone::Bottom
            } else {
                DropZone::Top
            });
        }
    }

    let expand = if outer_docking {
        0.0
    } else {
        (hs_w * 0.30).trunc()
    };
    let expand_rect = |r: Rect| -> Rect {
        Rect::new(
            Point::new(Px(r.origin.x.0 - expand), Px(r.origin.y.0 - expand)),
            Size::new(
                Px(r.size.width.0 + expand * 2.0),
                Px(r.size.height.0 + expand * 2.0),
            ),
        )
    };

    let mut picked: Option<DropZone> = None;
    for (zone, r) in dock_hint_rects_with_font(rect, font_size, outer_docking) {
        let hit = if outer_docking {
            r.contains(position)
        } else {
            expand_rect(r).contains(position)
        };
        if hit {
            picked = Some(zone);
        }
    }

    picked
}

pub(super) fn dock_hint_rects_with_font(
    rect: Rect,
    font_size: Px,
    outer_docking: bool,
) -> [(DropZone, Rect); 5] {
    // Align with Dear ImGui docking branch mental model:
    // - compute a 5-way “direction pad” around the center of the hovered dock node,
    // - with sizing derived from font size and panel size,
    // - and a distinct geometry for "outer docking" (bigger targets spaced further out).
    //
    // Reference:
    // - `repo-ref/imgui/imgui.cpp`: `DockNodeCalcDropRectsAndTestMousePos(...)`
    let parent_smaller_axis = rect.size.width.0.min(rect.size.height.0);
    let font = font_size.0.max(0.0);
    let hs_for_central_nodes = (font * 1.5).min((font * 0.5).max(parent_smaller_axis / 8.0));

    let (hs_w, hs_h, off_x, off_y) = if outer_docking {
        let hs_w = (hs_for_central_nodes * 1.50).trunc();
        let hs_h = (hs_for_central_nodes * 0.80).trunc();
        let off_x = (rect.size.width.0 * 0.5 - hs_h).trunc();
        let off_y = (rect.size.height.0 * 0.5 - hs_h).trunc();
        (hs_w, hs_h, off_x, off_y)
    } else {
        let hs_w = hs_for_central_nodes.trunc();
        let hs_h = (hs_for_central_nodes * 0.90).trunc();
        let off = (hs_w * 2.40).trunc();
        (hs_w, hs_h, off, off)
    };

    let cx = (rect.origin.x.0 + rect.size.width.0 * 0.5).trunc();
    let cy = (rect.origin.y.0 + rect.size.height.0 * 0.5).trunc();

    let center = Rect::new(
        Point::new(Px(cx - hs_w), Px(cy - hs_w)),
        Size::new(Px(hs_w * 2.0), Px(hs_w * 2.0)),
    );
    let left = Rect::new(
        Point::new(Px(cx - off_x - hs_h), Px(cy - hs_w)),
        Size::new(Px(hs_h * 2.0), Px(hs_w * 2.0)),
    );
    let right = Rect::new(
        Point::new(Px(cx + off_x - hs_h), Px(cy - hs_w)),
        Size::new(Px(hs_h * 2.0), Px(hs_w * 2.0)),
    );
    let top = Rect::new(
        Point::new(Px(cx - hs_w), Px(cy - off_y - hs_h)),
        Size::new(Px(hs_w * 2.0), Px(hs_h * 2.0)),
    );
    let bottom = Rect::new(
        Point::new(Px(cx - hs_w), Px(cy + off_y - hs_h)),
        Size::new(Px(hs_w * 2.0), Px(hs_h * 2.0)),
    );

    [
        (DropZone::Center, center),
        (DropZone::Left, left),
        (DropZone::Right, right),
        (DropZone::Top, top),
        (DropZone::Bottom, bottom),
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
